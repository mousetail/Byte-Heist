use std::sync::atomic::AtomicUsize;

use common::AchievementType;
use serde::Deserialize;
use sqlx::{PgPool, query};
use time::OffsetDateTime;

#[derive(Deserialize, Debug)]
struct GitHubStarUser {
    id: i64,
}

#[derive(Deserialize, Debug)]
struct GithubStarsResponse {
    starred_at: String,
    user: GitHubStarUser,
}

#[derive(Deserialize, Debug)]
struct GithubContributorsResponse {
    id: i64,
}

async fn get_stargazers(
    client: &reqwest::Client,
    github_access_token: &str,
) -> Result<Vec<GithubStarsResponse>, AwardMiscAchievementsError> {
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
        return Err(AwardMiscAchievementsError::GithubReturnedBadStatusCode);
    }
    let data: Vec<GithubStarsResponse> = response.error_for_status()?.json().await?;
    Ok(data)
}

async fn get_contributors(
    client: &reqwest::Client,
    github_access_token: &str,
) -> Result<Vec<GithubContributorsResponse>, AwardMiscAchievementsError> {
    let response = client
        .get("https://api.github.com/repos/mousetail/byte-heist/contributors")
        .header("Accept", "Application/vnd.github+json")
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
        return Err(AwardMiscAchievementsError::GithubReturnedBadStatusCode);
    }
    let data: Vec<GithubContributorsResponse> = response.error_for_status()?.json().await?;
    Ok(data)
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
    let stargazers = get_stargazers(&client, &github_access_token).await?;

    let format = time::format_description::well_known::Rfc3339;
    for item in stargazers {
        insert_achievement_response(
            AchievementType::StarTheRepo,
            pool,
            item.user.id,
            OffsetDateTime::parse(&item.starred_at, &format)?,
        )
        .await?;
    }

    let contributors = get_contributors(&client, &github_access_token).await?;
    for item in contributors {
        insert_achievement_response(
            AchievementType::Contribute,
            pool,
            item.id,
            OffsetDateTime::now_utc(),
        )
        .await?;
    }

    Ok(())
}

async fn insert_achievement_response(
    achievement: AchievementType,
    pool: &PgPool,
    value: i64,
    date: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    let achievement_name: &str = achievement.into();

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
        value
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
    ParseDateError(#[allow(unused)] time::error::Parse),
    GithubReturnedBadStatusCode,
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

impl From<time::error::Parse> for AwardMiscAchievementsError {
    fn from(value: time::error::Parse) -> Self {
        Self::ParseDateError(value)
    }
}

pub(super) async fn award_misc_achievements(
    pool: &PgPool,
) -> Result<(), AwardMiscAchievementsError> {
    award_github_achievements(pool).await?;

    Ok(())
}
