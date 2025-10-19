use std::{borrow::Cow, collections::HashSet};

use axum::{Extension, extract::Path};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, query_as, query_scalar};

use crate::{
    controllers::challenges::suggest_changes::handle_diff,
    error::Error,
    models::{account::Account, challenge::NewOrExistingChallenge},
    tera_utils::auto_input::AutoInput,
    test_case_display::{DiffElement, OutputDisplay, get_diff_elements},
};

use super::{
    reactions::RawReaction,
    suggest_changes::{CommentDiff, DiffStatus},
};

#[derive(PartialEq, Eq)]
pub(super) struct RawComment {
    pub(super) id: i32,
    challenge_id: i32,
    author_id: i32,
    author_username: String,
    author_avatar: String,
    parent: Option<i32>,
    message: String,
    old_value: Option<String>,
    new_value: Option<String>,
    status: Option<DiffStatus>,
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
            r#"
                SELECT
                    challenge_comments.id,
                    challenge_comments.challenge as challenge_id,
                    parent,
                    message,
                    author as author_id,
                    accounts.username as author_username,
                    accounts.avatar as author_avatar,
                    challenge_change_suggestions.old_value as "old_value?",
                    challenge_change_suggestions.new_value as "new_value?",
                    challenge_change_suggestions.status as "status?: DiffStatus"
                FROM challenge_comments
                LEFT JOIN accounts on challenge_comments.author = accounts.id
                LEFT JOIN challenge_change_suggestions ON challenge_change_suggestions.comment = challenge_comments.id
                WHERE challenge_comments.challenge = $1
                ORDER BY id ASC
            "#,
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

#[derive(Serialize, Eq, PartialEq)]
struct ProcessedDiff {
    columns: (Vec<DiffElement>, Vec<DiffElement>),
    status: DiffStatus,
}

#[derive(Serialize, Eq, PartialEq)]
struct ProcessedComment {
    id: i32,
    parent: Option<i32>,
    message: String,
    children: Vec<ProcessedComment>,
    author_id: i32,
    author_username: String,
    author_avatar: String,

    up_reactions: HashSet<Reaction>,
    down_reactions: HashSet<Reaction>,

    diff: Option<ProcessedDiff>,
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
            .map(|e| {
                let diff = e
                    .old_value
                    .zip(e.new_value)
                    .map(|(left, right)| get_diff_elements(left, right, "\n"))
                    .zip(e.status)
                    .map(|(diff, status)| ProcessedDiff {
                        columns: diff,
                        status,
                    });
                ProcessedComment {
                    id: e.id,
                    parent: e.parent,
                    message: e.message,
                    children: vec![],

                    author_id: e.author_id,
                    author_avatar: e.author_avatar,
                    author_username: e.author_username,

                    up_reactions: HashSet::new(),
                    down_reactions: HashSet::new(),

                    diff,
                }
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
    diff: Option<CommentDiff>,
    parent: Option<i32>,
}

impl NewComment {
    async fn submit(&self, challenge: i32, author: i32, pool: &PgPool) -> Result<i32, sqlx::Error> {
        query_scalar!(
            "
            INSERT INTO challenge_comments(challenge, parent, author, message)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            ",
            challenge,
            self.parent,
            author,
            self.message
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
) -> Result<OutputDisplay, Error> {
    if !account.has_solved_a_challenge {
        return Err(Error::PermissionDenied(
            "Can't post a comment until your account has solved at least one challenge",
        ));
    }

    if data.message.is_empty() || data.message.len() > 5000 {
        return Err(Error::PermissionDenied(
            "Message can't be empty of more than 5kb",
        ));
    }

    // Sanity check if you are reacting to a comment that exists and is under the correct challenge
    if let Some(parent_id) = data.parent
        && RawComment::get_challenge_by_id(&pool, parent_id)
            .await
            .map_err(Error::Database)?
            != Some(id)
    {
        return Err(Error::ServerError);
    }

    let task = if let Some(diff) = &data.diff {
        if data.parent.is_some() {
            return Err(Error::PermissionDenied(
                "Can't submit a diff as a child comment",
            ));
        }

        match handle_diff(&pool, id, diff).await? {
            Ok(e) => Some(e),
            Err(d) => return Ok(d),
        }
    } else {
        None
    };

    // Only apply the rate limit if the validation succceeded
    account.rate_limit(&pool).await?;

    let result = data
        .submit(id, account.id, &pool)
        .await
        .map_err(Error::Database)?;

    if let Some(task) = task {
        task.apply(&pool, result).await?;
    }

    Err(Error::Redirect(Cow::Owned(format!(
        "/challenge/{id}/{slug}/view"
    ))))
}
