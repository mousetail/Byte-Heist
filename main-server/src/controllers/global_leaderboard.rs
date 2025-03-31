use axum::{extract::Path, Extension};
use serde::Serialize;
use sqlx::PgPool;

use crate::{
    error::Error,
    models::{challenge::ChallengeCategory, global_leaderboard::GlobalLeaderboardEntry},
};

#[derive(Serialize)]
pub struct GlobalLeaderboardOutput {
    entries: Vec<GlobalLeaderboardEntry>,
    category: ChallengeCategory,
}

pub async fn global_leaderboard(
    Extension(pool): Extension<PgPool>,
    Path(category): Path<ChallengeCategory>,
) -> Result<GlobalLeaderboardOutput, Error> {
    let data = GlobalLeaderboardEntry::get_all(&pool, category).await?;
    Ok(GlobalLeaderboardOutput {
        entries: data,
        category,
    })
}
