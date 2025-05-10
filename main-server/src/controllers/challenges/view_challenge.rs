use axum::{extract::Path, Extension};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, query_scalar, PgPool};

use crate::{error::Error, models::{account::Account, challenge::NewOrExistingChallenge}, tera_utils::auto_input::AutoInput};

#[derive(PartialEq, Eq)]
struct RawComment {
    id: i32,
    challenge_id: i32,
    author_id: i32,
    author_username: String,
    author_avatar: String,
    parent: Option<i32>,
    message: String,
    diff: Option<String>
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
    async fn get_raw_comments_for_challenge(pool: &PgPool, challenge_id: i32) -> Result<Vec<RawComment>, sqlx::Error> {
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
        ).fetch_all(pool).await
    }
}

#[derive(Serialize, Eq, PartialEq, Ord, PartialOrd)]
struct ProcessedComment {
    id: i32,
    parent: Option<i32>,
    message: String,
    diff: Option<String>,
    children: Vec<ProcessedComment>,
    author_id: i32,
    author_username: String,
    author_avatar: String
}

impl ProcessedComment {
    fn sort(&mut self) {
        self.children.sort();

        for child in &mut self.children {
            child.sort();
        }
    }

    fn from_raw_comments(comments: Vec<RawComment>) -> Vec<ProcessedComment> {
        let mut output: Vec<_> = comments.into_iter().map(
            |e| ProcessedComment {
                id: e.id,
                parent: e.parent,
                message: e.message,
                diff: e.diff,
                children: vec![],

                author_id: e.author_id,
                author_avatar: e.author_avatar,
                author_username: e.author_username
            }
        ).collect();

        let mut index = output.len();

        while index > 0 {
            if let Some(parent) = output[index - 1].parent {
                let value = output.swap_remove(index - 1);

                let parent_index = output.binary_search_by_key(&parent, |d| d.id).unwrap();
                output[parent_index].children.push(value);
            }
            index-=1;
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
    comments: Vec<ProcessedComment>
}

pub async fn view_challenge(
    Path((id, _slug)): Path<(i32, String)>,
    Extension(pool): Extension<PgPool>,
) -> Result<ViewChallengeOutput, Error> {
    let comments = ProcessedComment::from_raw_comments(
        RawComment::get_raw_comments_for_challenge(&pool, id).await.map_err(
            Error::Database
        )?
    );

    Ok(ViewChallengeOutput {
        challenge: NewOrExistingChallenge::get_by_id(&pool, id)
        .await?
        .ok_or(Error::NotFound)?,
        comments
    })
}

#[derive(Deserialize)]
pub struct NewComment {
    message: String,
    diff: Option<String>,
    parent: Option<i32>
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
        ).fetch_one(pool).await
    }
}

pub async fn post_comment(
    Path((id, slug)): Path<(i32, String)>, 
    account: Account,
    Extension(pool): Extension<PgPool>,
    data: AutoInput<NewComment>
) -> Result<ViewChallengeOutput, Error> {
    let _result = data.0.submit(id    , account.id, &pool).await.map_err(Error::Database)?;

    view_challenge(Path((id, slug)), Extension(pool)).await
}