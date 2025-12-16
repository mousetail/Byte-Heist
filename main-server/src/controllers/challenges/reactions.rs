use std::borrow::Cow;

use super::{
    suggest_changes::{DiffField, DiffStatus},
    view_challenge::RawComment,
};
use axum::{Extension, extract::Path};
use serde::Deserialize;
use sqlx::{PgPool, query, query_as, query_scalar};

use crate::{
    achievements::{award_achievement, vote_achievements::award_vote_achievements},
    controllers::challenges::suggest_changes::CommentDiff,
    error::Error,
    models::account::Account,
    tera_utils::auto_input::AutoInput,
};

pub(super) struct RawChallengeReaction {
    pub author_id: i32,
    pub author_username: String,
    pub is_upvote: bool,
}

impl RawChallengeReaction {
    pub(super) async fn get_for_challenge(
        pool: &PgPool,
        challenge_id: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        query_as!(
            Self,
            r#"
                SELECT author as author_id, is_upvote, accounts.username as author_username FROM challenge_votes            
                INNER JOIN accounts ON accounts.id = challenge_votes.author
                WHERE challenge=$1
                ORDER BY is_upvote
            "#,
            challenge_id
        ).fetch_all(pool).await
    }
}

pub(super) struct RawCommentReaction {
    pub comment_id: i32,
    pub author_id: i32,
    pub author_username: String,
    pub is_upvote: bool,
}

impl RawCommentReaction {
    async fn get_reactions_for_comment(
        pool: &PgPool,
        comment_id: i32,
    ) -> Result<Vec<RawCommentReaction>, sqlx::Error> {
        query_as!(
            RawCommentReaction,
            "
                SELECT 
                    comment as comment_id,
                    author as author_id,
                    is_upvote,
                    accounts.username as author_username
                FROM challenge_comment_votes
                INNER JOIN accounts ON accounts.id = challenge_comment_votes.author
                WHERE challenge_comment_votes.comment = $1
                ORDER BY comment ASC
            ",
            comment_id
        )
        .fetch_all(pool)
        .await
    }

    pub(super) async fn get_reactions_for_comments(
        pool: &PgPool,
        comments: &[RawComment],
    ) -> Result<Vec<RawCommentReaction>, sqlx::Error> {
        query_as!(
            RawCommentReaction,
            "
                SELECT
                    comment as comment_id,
                    author as author_id,
                    is_upvote,
                    accounts.username as author_username
                FROM challenge_comment_votes
                INNER JOIN accounts ON accounts.id = challenge_comment_votes.author
                WHERE challenge_comment_votes.comment = ANY($1)
                ORDER BY comment ASC
            ",
            &comments.iter().map(|i| i.id).collect::<Vec<_>>()
        )
        .fetch_all(pool)
        .await
    }
}

#[derive(Deserialize)]
pub struct NewReaction {
    comment_id: Option<i32>,
    is_upvote: bool,
}

impl NewReaction {
    async fn submit(
        &self,
        author: i32,
        pool: &PgPool,
        challenge_id: i32,
    ) -> Result<i32, sqlx::Error> {
        if let Some(comment_id) = self.comment_id {
            query!(
                "
            UPDATE challenge_comments
            SET last_vote_time=now()
            WHERE id=$1
            ",
                comment_id
            )
            .execute(pool)
            .await?;

            query_scalar!(
                "
            INSERT INTO challenge_comment_votes(
                author,
                comment,
                is_upvote
            )
            VALUES ($1, $2, $3)
            ON CONFLICT(author, comment) DO UPDATE SET is_upvote=$3
            RETURNING id
            ",
                author,
                comment_id,
                self.is_upvote
            )
            .fetch_one(pool)
            .await
        } else {
            query_scalar!(
                r#"
                    INSERT INTO challenge_votes(
                        author,
                        is_upvote,
                        challenge
                    )
                    VALUES ($1, $2, $3)
                    ON CONFLICT(author, challenge) DO UPDATE SET is_upvote=$2
                    RETURNING id
                "#,
                author,
                self.is_upvote,
                challenge_id
            )
            .fetch_one(pool)
            .await
        }
    }
}

pub async fn post_reaction(
    Path((id, slug)): Path<(i32, String)>,
    account: Account,
    Extension(pool): Extension<PgPool>,
    AutoInput(reaction): AutoInput<NewReaction>,
) -> Result<(), Error> {
    reaction
        .submit(account.id, &pool, id)
        .await
        .map_err(Error::Database)?;

    Err(Error::Redirect(
        crate::error::RedirectType::TemporaryGet,
        Cow::Owned(format!("/challenge/{id}/{slug}/view")),
    ))
}

pub async fn handle_reactions(pool: &PgPool) -> Result<(), sqlx::Error> {
    struct CommentInfo {
        id: i32,
        challenge_id: i32,
        author_id: i32,
    }

    let messages = query_as!(
        CommentInfo,
        r#"
            SELECT id, challenge as challenge_id, author as author_id
            FROM challenge_comments
            WHERE last_vote_time > last_vote_processed_time
        "#
    )
    .fetch_all(pool)
    .await?;

    for CommentInfo {
        id: comment_id,
        challenge_id,
        author_id,
    } in messages
    {
        let diff = query_as!(
            CommentDiff,
            r#"
            SELECT
                field as "field: DiffField",
                new_value as replacement_value
            FROM challenge_change_suggestions
            WHERE comment=$1
            AND status='active'
            "#,
            comment_id
        )
        .fetch_optional(pool)
        .await?;

        if let Some(diff) = diff {
            process_diff(pool, diff, challenge_id, comment_id, author_id).await?;
        }

        query!(
            "UPDATE challenge_comments
            SET last_vote_processed_time=now()
            WHERE id=$1",
            comment_id
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn process_diff(
    pool: &PgPool,
    diff: CommentDiff,
    challenge_id: i32,
    comment_id: i32,
    author_id: i32,
) -> Result<(), sqlx::Error> {
    let reactions = RawCommentReaction::get_reactions_for_comment(pool, comment_id).await?;

    for reaction in &reactions {
        award_achievement(
            pool,
            reaction.author_id,
            common::AchievementType::Vote,
            Some(challenge_id),
            None,
        )
        .await?;
    }

    let up_reactions = reactions.iter().filter(|i| i.is_upvote).count();
    let down_reactions = reactions.len() - up_reactions;

    let status = if up_reactions.saturating_sub(down_reactions) > 2 && down_reactions < 2 {
        diff.apply(pool, challenge_id, comment_id, author_id)
            .await?;

        DiffStatus::Accepted
    } else if down_reactions.saturating_sub(up_reactions) > 2 && up_reactions < 2 {
        DiffStatus::Rejected
    } else {
        DiffStatus::Active
    };

    if status != DiffStatus::Active {
        for reaction in reactions {
            award_vote_achievements(pool, reaction.author_id).await?;
        }

        query!(
            "
                UPDATE challenge_change_suggestions
                SET status=$1
                WHERE comment=$2
            ",
            status as DiffStatus,
            comment_id
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
