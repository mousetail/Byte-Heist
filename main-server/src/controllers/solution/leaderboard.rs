use axum::{
    extract::{Path, Query},
    Extension,
};
use sqlx::PgPool;

use crate::{
    error::Error,
    models::{account::Account, solutions::LeaderboardEntry},
};

use super::SolutionQueryParameters;

pub async fn get_leaderboard(
    Path((challenge_id, _slug, language_name)): Path<(i32, String, String)>,
    Query(SolutionQueryParameters { ranking }): Query<SolutionQueryParameters>,
    account: Account,
    Extension(pool): Extension<PgPool>,
) -> Result<Vec<LeaderboardEntry>, Error> {
    let leaderbaord = LeaderboardEntry::get_leaderboard_near(
        &pool,
        challenge_id,
        &language_name,
        Some(account.id),
        ranking,
    )
    .await
    .map_err(Error::Database)?;

    Ok(leaderbaord)
}
