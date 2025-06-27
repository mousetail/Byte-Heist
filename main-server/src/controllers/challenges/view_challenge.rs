use std::{borrow::Cow, collections::HashSet};

use axum::{extract::Path, Extension};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, query_scalar, PgPool};

use crate::{
    error::Error,
    models::{account::Account, challenge::NewOrExistingChallenge},
    tera_utils::auto_input::AutoInput,
};

#[derive(PartialEq, Eq)]
struct RawComment {
    id: i32,
    challenge_id: i32,
    author_id: i32,
    author_username: String,
    author_avatar: String,
    parent: Option<i32>,
    message: String,
    diff: Option<String>,
}

impl PartialOrd for RawComment {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RawComment {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl RawComment {
    async fn get_raw_comments_for_challenge(
        pool: &PgPool,
        challenge_id: i32,
    ) -> Result<Vec<RawComment>, sqlx::Error> {
        query_as!(
            RawComment,
            "
                SELECT
                    challenge_comments.id,
                    challenge as challenge_id,
                    parent,
                    message,
                    diff,
                    author as author_id,
                    accounts.username as author_username,
                    accounts.avatar as author_avatar
                FROM challenge_comments
                LEFT JOIN accounts on challenge_comments.author = accounts.id
                WHERE challenge = $1
                ORDER BY id ASC
            ",
            challenge_id
        )
        .fetch_all(pool)
        .await
    }

    async fn get_challenge_by_id(pool: &PgPool, id: i32) -> Result<Option<i32>, sqlx::Error> {
        query_scalar!(
            "
                SELECT challenge
                FROM challenge_comments
                WHERE id=$1
            ",
            id
        )
        .fetch_optional(pool)
        .await
    }
}

struct RawReaction {
    #[allow(unused)]
    id: i32,
    comment_id: i32,
    author_id: i32,
    author_username: String,
    is_upvote: bool,
}

impl RawReaction {
    async fn get_reactions_for_challenge(
        pool: &PgPool,
        comments: &[RawComment],
    ) -> Result<Vec<RawReaction>, sqlx::Error> {
        query_as!(
            RawReaction,
            "
                SELECT challenge_comment_votes.id,
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

#[derive(Serialize, Eq, PartialEq)]
struct ProcessedComment {
    id: i32,
    parent: Option<i32>,
    message: String,
    diff: Option<String>,
    children: Vec<ProcessedComment>,
    author_id: i32,
    author_username: String,
    author_avatar: String,

    up_reactions: HashSet<Reaction>,
    down_reactions: HashSet<Reaction>,
}

impl Ord for ProcessedComment {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for ProcessedComment {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Serialize, PartialEq, Eq, Hash)]
struct Reaction {
    author_id: i32,
    author_username: String,
}

impl Reaction {
    /// Assumes reactions is sorted by comment id
    /// Assumes comments is sorted by id
    fn apply_reactions_to_comments(comments: &mut [ProcessedComment], reactions: Vec<RawReaction>) {
        let mut first_index = 0;
        for reaction in reactions {
            let index = comments[first_index..]
                .binary_search_by_key(&reaction.comment_id, |e| e.id)
                .unwrap()
                + first_index;

            let set = match reaction.is_upvote {
                true => &mut comments[index].up_reactions,
                false => &mut comments[index].down_reactions,
            };
            set.insert(Reaction {
                author_id: reaction.author_id,
                author_username: reaction.author_username,
            });

            first_index = index;
        }
    }
}

impl ProcessedComment {
    fn sort(&mut self) {
        self.children.sort();

        for child in &mut self.children {
            child.sort();
        }
    }

    fn from_raw_comments(
        comments: Vec<RawComment>,
        reactions: Vec<RawReaction>,
    ) -> Vec<ProcessedComment> {
        let mut output: Vec<_> = comments
            .into_iter()
            .map(|e| ProcessedComment {
                id: e.id,
                parent: e.parent,
                message: e.message,
                diff: e.diff,
                children: vec![],

                author_id: e.author_id,
                author_avatar: e.author_avatar,
                author_username: e.author_username,

                up_reactions: HashSet::new(),
                down_reactions: HashSet::new(),
            })
            .collect();

        Reaction::apply_reactions_to_comments(&mut output, reactions);

        let mut index = output.len();

        while index > 0 {
            if let Some(parent) = output[index - 1].parent {
                let value = output.swap_remove(index - 1);

                let parent_index = output.binary_search_by_key(&parent, |d| d.id).unwrap();
                output[parent_index].children.push(value);
            }
            index -= 1;
        }

        output.sort();
        for comment in &mut output {
            comment.sort();
        }

        output
    }
}

#[derive(Serialize)]
pub struct ViewChallengeOutput {
    #[serde(flatten)]
    challenge: NewOrExistingChallenge,
    comments: Vec<ProcessedComment>,
}

pub async fn view_challenge(
    Path((id, _slug)): Path<(i32, String)>,
    Extension(pool): Extension<PgPool>,
) -> Result<ViewChallengeOutput, Error> {
    let raw_comments = RawComment::get_raw_comments_for_challenge(&pool, id)
        .await
        .map_err(Error::Database)?;
    let reactions = RawReaction::get_reactions_for_challenge(&pool, &raw_comments)
        .await
        .map_err(Error::Database)?;
    let comments = ProcessedComment::from_raw_comments(raw_comments, reactions);

    Ok(ViewChallengeOutput {
        challenge: NewOrExistingChallenge::get_by_id(&pool, id)
            .await?
            .ok_or(Error::NotFound)?,
        comments,
    })
}

#[derive(Deserialize)]
pub struct NewComment {
    message: String,
    diff: Option<String>,
    parent: Option<i32>,
}

impl NewComment {
    async fn submit(&self, challenge: i32, author: i32, pool: &PgPool) -> Result<i32, sqlx::Error> {
        query_scalar!(
            "
            INSERT INTO challenge_comments(challenge, parent, author, message, diff)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            ",
            challenge,
            self.parent,
            author,
            self.message,
            self.diff
        )
        .fetch_one(pool)
        .await
    }
}

pub async fn post_comment(
    Path((id, slug)): Path<(i32, String)>,
    account: Account,
    Extension(pool): Extension<PgPool>,
    AutoInput(data): AutoInput<NewComment>,
) -> Result<(), Error> {
    if !account.has_solved_a_challenge {
        return Err(Error::PermissionDenied(
            "Can't post a comment until your account has solved at least one challenge",
        ));
    }

    account.rate_limit(&pool).await?;

    if data.message.is_empty() || data.message.len() > 5000 {
        return Err(Error::PermissionDenied(
            "Message can't be empty of more than 5kb",
        ));
    }

    // Sanity check if you are reacting to a comment that exists and is under the correct challenge
    if let Some(parent_id) = data.parent {
        if RawComment::get_challenge_by_id(&pool, parent_id)
            .await
            .map_err(Error::Database)?
            != Some(id)
        {
            return Err(Error::ServerError);
        }
    }

    let _result = data
        .submit(id, account.id, &pool)
        .await
        .map_err(Error::Database)?;

    Err(Error::Redirect(Cow::Owned(format!(
        "/challenge/{id}/{slug}/view"
    ))))
}

#[derive(Deserialize)]
pub struct NewReaction {
    comment_id: i32,
    is_upvote: bool,
}

impl NewReaction {
    async fn submit(&self, author: i32, pool: &PgPool) -> Result<i32, sqlx::Error> {
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
            self.comment_id,
            self.is_upvote
        )
        .fetch_one(pool)
        .await
    }
}

pub async fn post_reaction(
    Path((id, slug)): Path<(i32, String)>,
    account: Account,
    Extension(pool): Extension<PgPool>,
    AutoInput(reaction): AutoInput<NewReaction>,
) -> Result<(), Error> {
    reaction
        .submit(account.id, &pool)
        .await
        .map_err(Error::Database)?;

    Err(Error::Redirect(Cow::Owned(format!(
        "/challenge/{id}/{slug}/view"
    ))))
}
