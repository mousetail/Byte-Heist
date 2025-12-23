use crate::{error::Error, models::account::Account};
use axum::Extension;
use common::diff_tools::{Columns, get_diff_elements};
use serde::Serialize;
use sqlx::{PgPool, query_as, query_scalar};

struct RawPendingChangeSuggestion {
    id: i32,
    challenge_id: i32,
    challenge_name: String,
    author_id: i32,
    author_name: String,
    author_avatar: String,
    message: String,
    old_value: String,
    new_value: String,
}

impl RawPendingChangeSuggestion {
    async fn get_all(pool: &PgPool, account_id: Option<i32>) -> Result<Vec<Self>, sqlx::Error> {
        query_as!(
            Self,
            r#"
                SELECT
                    challenge_change_suggestions.comment as id,
                    challenge_comments.message as message,
                    challenges.id as challenge_id,
                    challenges.name as challenge_name,
                    accounts.id as author_id,
                    accounts.username as author_name,
                    accounts.avatar as author_avatar,
                    challenge_change_suggestions.old_value as old_value,
                    challenge_change_suggestions.new_value as new_value
                FROM challenge_change_suggestions
                INNER JOIN challenge_comments ON challenge_comments.id = challenge_change_suggestions.comment
                INNER JOIN challenges ON challenge_comments.challenge=challenges.id
                INNER JOIN accounts ON challenge_comments.author=accounts.id
                WHERE
                    challenge_change_suggestions.status='active'
                    AND (challenges.category != 'private' OR challenges.author=$1)
                    AND (
                        $1::integer IS NULL
                        OR NOT EXISTS(
                            SELECT FROM challenge_comment_votes
                            WHERE challenge_comment_votes.comment = challenge_change_suggestions.comment
                            AND challenge_comment_votes.author=$1
                        )
                    )
            "#,
            account_id
        ).fetch_all(pool).await
    }
}

#[derive(Serialize)]
pub struct ProcessedPendingChangeSuggestion {
    id: i32,
    challenge_id: i32,
    challenge_name: String,
    author_id: i32,
    author_name: String,
    author_avatar: String,
    message: String,
    diff: Columns,
}

impl ProcessedPendingChangeSuggestion {
    fn from_raw(source: RawPendingChangeSuggestion) -> Self {
        let diff = get_diff_elements(&source.old_value, &source.new_value, "\n", 0);

        Self {
            diff,
            id: source.id,
            challenge_id: source.challenge_id,
            challenge_name: source.challenge_name,
            author_id: source.author_id,
            author_name: source.author_name,
            author_avatar: source.author_avatar,
            message: source.message,
        }
    }
}

pub async fn get_pending_change_suggestions(
    account: Option<Account>,
    Extension(pool): Extension<PgPool>,
) -> Result<Vec<ProcessedPendingChangeSuggestion>, Error> {
    Ok(
        RawPendingChangeSuggestion::get_all(&pool, account.map(|i| i.id))
            .await
            .map_err(Error::Database)?
            .into_iter()
            .map(ProcessedPendingChangeSuggestion::from_raw)
            .collect(),
    )
}

pub async fn get_unread_change_suggestions_for_user(
    pool: &PgPool,
    user_id: i32,
) -> Result<i64, sqlx::Error> {
    query_scalar!(
        "
            SELECT COUNT(*)
            FROM challenge_change_suggestions
            INNER JOIN challenges ON challenges.id=challenge_change_suggestions.challenge
            WHERE challenge_change_suggestions.status='active'
            AND (challenges.category != 'private' OR challenges.author=$1)
            AND NOT EXISTS(
                SELECT FROM challenge_comment_votes
                    WHERE challenge_comment_votes.comment = challenge_change_suggestions.comment
                    AND challenge_comment_votes.author=$1
            )
        ",
        user_id
    )
    .fetch_one(pool)
    .await
    .map(Option::unwrap_or_default)
}
