use axum::{extract::Path, Extension};
use serde::Serialize;
use sqlx::PgPool;

use crate::{
    auto_output_format::{AutoOutputFormat, Format},
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
    format: Format,
    Path(category): Path<ChallengeCategory>,
) -> Result<AutoOutputFormat<GlobalLeaderboardOutput>, Error> {
    let data = GlobalLeaderboardEntry::get_all(&pool, category).await?;
    Ok(AutoOutputFormat::new(
        GlobalLeaderboardOutput {
            entries: data,
            category,
        },
        "global_leaderboard.html.jinja",
        format,
    ))
}
