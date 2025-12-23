use axum::{Extension, extract::Path};
use common::sql_enums::ChallengeCategory;
use serde::Serialize;
use sqlx::PgPool;

use crate::{error::Error, models::global_leaderboard::GlobalLeaderboardEntry};

#[derive(Serialize)]
pub struct GlobalLeaderboardOutput {
    entries: Vec<GlobalLeaderboardEntry>,
    category: ChallengeCategory,
    language: Option<String>,
}

pub async fn global_leaderboard(
    Extension(pool): Extension<PgPool>,
    Path(category): Path<ChallengeCategory>,
) -> Result<GlobalLeaderboardOutput, Error> {
    let data = GlobalLeaderboardEntry::get_all(&pool, category).await?;
    Ok(GlobalLeaderboardOutput {
        entries: data,
        category,
        language: None,
    })
}

pub async fn global_leaderboard_per_language(
    Extension(pool): Extension<PgPool>,
    Path((category, language)): Path<(ChallengeCategory, String)>,
) -> Result<GlobalLeaderboardOutput, Error> {
    let data = GlobalLeaderboardEntry::get_all_by_language(&pool, category, &language).await?;
    Ok(GlobalLeaderboardOutput {
        entries: data,
        category,
        language: Some(language),
    })
}
