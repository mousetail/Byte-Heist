use axum::{extract::Path, Extension};
use serde::Serialize;
use sqlx::{query_as, types::time::OffsetDateTime, PgPool};

use crate::{
    error::Error,
    models::{account::Account, solutions::InvalidatedSolution},
};

#[derive(Serialize)]
pub struct UserPageLeaderboardEntry {
    language: String,
    score: i32,
    challenge_id: i32,
    challenge_name: String,
}

#[derive(Serialize)]
pub struct AccountProfileInfo {
    username: String,
    avatar: String,
    join_date: OffsetDateTime,
    solutions: Option<i64>,
    distinct_challenges: Option<i64>,
    first_places: Option<i64>,
    top_ten_percents: Option<i64>,
    rank: Option<i64>,
}

#[derive(Serialize)]
pub struct UserInfo {
    account_info: AccountProfileInfo,
    solutions: Vec<UserPageLeaderboardEntry>,
    invalidated_solutions: Option<Vec<InvalidatedSolution>>,
    id: i32,
}

pub async fn get_user(
    Path(id): Path<i32>,
    account: Option<Account>,
    Extension(pool): Extension<PgPool>,
) -> Result<UserInfo, Error> {
    let account_info = query_as!(
        AccountProfileInfo,
        r#"
            SELECT
                username,
                avatar,
                user_scoring_info.sols as "solutions",
                user_scoring_info.first_places as "first_places",
                user_scoring_info.top_ten_percents as "top_ten_percents",
                user_scoring_info.distinct_challenges as "distinct_challenges",
                user_scoring_info.rank as "rank",
                created_at as join_date
            FROM accounts
            LEFT JOIN user_scoring_info
            ON accounts.id = user_scoring_info.author
            WHERE id=$1 AND (
                category IS NULL OR
                category='code-golf'
            )
        "#,
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(Error::Database)?;
    let Some(account_info) = account_info else {
        return Err(Error::NotFound);
    };

    let invalidated_solutions = match account {
        Some(acc) if acc.id == id => Some(
            InvalidatedSolution::get_invalidated_solutions_for_user(id, &pool)
                .await
                .map_err(Error::Database)?,
        ),
        _ => None,
    };

    let solutions = query_as!(
        UserPageLeaderboardEntry,
        "SELECT solutions.language, solutions.score, solutions.challenge as challenge_id, challenges.name as challenge_name
        FROM solutions
        LEFT JOIN challenges ON challenges.id = solutions.challenge
        WHERE solutions.author=$1
        AND solutions.valid=true
        AND challenges.status in ('public', 'beta')",
        id
    ).fetch_all(&pool).await
    .map_err(Error::Database)?;

    Ok(UserInfo {
        solutions,
        account_info,
        id,
        invalidated_solutions,
    })
}
