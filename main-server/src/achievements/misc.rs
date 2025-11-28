use std::sync::atomic::AtomicUsize;

use serde::Deserialize;
use sqlx::{PgPool, query};
use time::OffsetDateTime;

use crate::achievements::AchievementType;

#[derive(Deserialize, Debug)]
struct GitHubStarUser {
    id: i64,
}

#[derive(Deserialize, Debug)]
struct GithubStarsResponse {
    starred_at: String,
    user: GitHubStarUser,
}

async fn award_github_achievements(pool: &PgPool) -> Result<(), AwardMiscAchievementsError> {
    static ATTEMPT_NUMBER: AtomicUsize = AtomicUsize::new(0);

    let previous_attempt_number = ATTEMPT_NUMBER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    // Only fetch github stars every 7 "cycles"
    if !previous_attempt_number.is_multiple_of(7) {
        return Ok(());
    }

    let github_access_token = std::env::var("GITHUB_ACCESS_TOKEN")
        .expect("Missing the GITHUB_ACCESS_TOKEN environment variable.");

    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/repos/mousetail/byte-heist/stargazers")
        .header("Accept", "Application/vnd.github.star+json")
        .header("Authorization", format!("Bearer {github_access_token}"))
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header(
            "User-Agent",
            "Rust-Reqwest (Byte Heist https://byte-heist.com)",
        )
        .send()
        .await?;

    if !response.status().is_success() {
        let text = response.text().await?;
        eprintln!("Got error from GitHub:\n{text}");
        return Ok(());
    }
    let data: Vec<GithubStarsResponse> = response.error_for_status()?.json().await?;

    for item in data {
        insert_star_response(pool, item).await?;
    }

    Ok(())
}

async fn insert_star_response(
    pool: &PgPool,
    value: GithubStarsResponse,
) -> Result<(), sqlx::Error> {
    let achievement_name: &str = AchievementType::StarTheRepo.into();
    let format = &time::format_description::well_known::Rfc3339;
    let date = OffsetDateTime::parse(&value.starred_at, format)
        .expect("Invalid date returned from GitHub");

    query!(
        r#"
            INSERT INTO achievements(
                achievement, awarded_at, achieved, user_id
            ) SELECT
                $1,
                $2,
                true,
                account
            FROM account_oauth_codes
                WHERE account_oauth_codes.id_on_provider=$3
            ON CONFLICT DO NOTHING
        "#,
        achievement_name,
        date,
        value.user.id
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[derive(Debug)]
pub enum AwardMiscAchievementsError {
    Sqlx(#[allow(unused)] sqlx::Error),
    Http(#[allow(unused)] reqwest::Error),
    Serde(#[allow(unused)] serde_json::Error),
}

impl From<sqlx::Error> for AwardMiscAchievementsError {
    fn from(value: sqlx::Error) -> Self {
        Self::Sqlx(value)
    }
}

impl From<reqwest::Error> for AwardMiscAchievementsError {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}

impl From<serde_json::Error> for AwardMiscAchievementsError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

pub(super) async fn award_misc_achievements(
    pool: &PgPool,
) -> Result<(), AwardMiscAchievementsError> {
    award_github_achievements(pool).await?;

    Ok(())
}
