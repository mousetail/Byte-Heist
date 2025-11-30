use std::borrow::Cow;

use crate::models::challenge::ChallengeCategory;
use axum::{Extension, extract::Path};
use common::slug::Slug;
use serde::Serialize;
use sqlx::{PgPool, query_as, types::time::OffsetDateTime};

use crate::{
    error::Error,
    models::{account::Account, solutions::InvalidatedSolution},
};

#[derive(Serialize)]
pub struct UserPageLeaderboardEntry {
    language: String,
    points: i32,
    challenge_id: i32,
    challenge_name: String,
}

#[derive(Serialize)]
pub struct AccountProfileInfo {
    pub username: String,
    avatar: String,
    join_date: OffsetDateTime,
    solutions: Option<i64>,
    distinct_challenges: Option<i64>,
    first_places: Option<i64>,
    top_ten_percents: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ranks: Option<Vec<i64>>,
    categories: Option<Vec<ChallengeCategory>>,
}

impl AccountProfileInfo {
    pub async fn fetch(pool: &PgPool, user_id: i32) -> Result<Option<Self>, sqlx::Error> {
        query_as!(
            AccountProfileInfo,
            r#"
                WITH total_scoring AS (
                    SELECT
                        author as id,
                        CAST(SUM(sols) as BIGINT) as "sols",
                        CAST(SUM(first_places) as BIGINT) as "first_places",
                        CAST(SUM(top_ten_percents) as BIGINT) as "top_ten_percents",
                        CAST(SUM(distinct_challenges) as BIGINT) as "distinct_challenges",
                        ARRAY_AGG(rank ORDER BY rank DESC) as "ranks",
                        ARRAY_AGG(category ORDER BY rank DESC) as "categories"
                    FROM user_scoring_info
                    WHERE author=$1
                    GROUP BY author
                )
                SELECT
                    username,
                    avatar,
                    created_at as join_date,
                    total_scoring.sols as "solutions",
                    total_scoring.first_places as "first_places",
                    total_scoring.top_ten_percents as "top_ten_percents",
                    total_scoring.distinct_challenges as "distinct_challenges",
                    total_scoring.ranks as "ranks",
                    total_scoring.categories as "categories:Vec<ChallengeCategory>"
                FROM accounts
                LEFT JOIN total_scoring
                ON accounts.id = total_scoring.id
                WHERE accounts.id=$1
            "#,
            user_id
        )
        .fetch_optional(pool)
        .await
    }
}

#[derive(Serialize)]
pub struct UserInfo {
    account_info: AccountProfileInfo,
    solutions: Vec<UserPageLeaderboardEntry>,
    invalidated_solutions: Option<Vec<InvalidatedSolution>>,
    per_language_stats: Vec<StatsForLanguage>,
    per_category_stats: Vec<StatsPerCategory>,
    recent_achievements: Vec<UserAchievement>,
    id: i32,
}

#[derive(Serialize)]
struct StatsForLanguage {
    language: String,
    total_score: i64,
}

async fn get_account_language_stats(
    pool: &PgPool,
    user_id: i32,
) -> Result<Vec<StatsForLanguage>, sqlx::Error> {
    query_as!(
        StatsForLanguage,
        r#"
        SELECT
            language as "language!",
            CAST(sum(user_scoring_info_per_language.total_score) AS bigint) as "total_score!"
        FROM user_scoring_info_per_language
        WHERE author = $1
        GROUP BY user_scoring_info_per_language.language
        ORDER BY "total_score!" DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
}

#[derive(Serialize)]
struct StatsPerCategory {
    total_score: i64,
    category: ChallengeCategory,
}

async fn get_account_category_stats(
    pool: &PgPool,
    user_id: i32,
) -> Result<Vec<StatsPerCategory>, sqlx::Error> {
    query_as!(
        StatsPerCategory,
        r#"
        SELECT
            category as "category!: ChallengeCategory",
            CAST(sum(user_scoring_info.total_score) AS bigint) as "total_score!"
        FROM user_scoring_info
        WHERE author = $1
        GROUP BY user_scoring_info.category
        ORDER BY "total_score!" DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
}

#[derive(Serialize)]
struct UserAchievement {
    achievement: String,
    awarded_at: Option<OffsetDateTime>,
    progress: Option<i64>,
    total: Option<i64>,
}

async fn get_user_achievements(
    pool: &PgPool,
    user_id: i32,
    is_self: bool,
) -> Result<Vec<UserAchievement>, sqlx::Error> {
    let achieved_achievements = query_as!(
        UserAchievement,
        r#"
            SELECT achievement,
            awarded_at,
            progress,
            total
            FROM achievements
            WHERE user_id=$1
            ORDER BY achieved DESC, progress DESC, awarded_at DESC
            LIMIT 3
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    if is_self && achieved_achievements.iter().all(|i| i.awarded_at.is_some()) {
        let most_progress_achievement = query_as!(
            UserAchievement,
            r#"
                SELECT achievement,
                awarded_at,
                progress,
                total
                From achievements
                WHERE user_id=$1 and not achieved
                ORDER BY progress DESC, total ASC
                LIMIT 1
            "#,
            user_id
        )
        .fetch_optional(pool)
        .await?;

        return Ok(most_progress_achievement
            .into_iter()
            .chain(achieved_achievements)
            .take(3)
            .collect());
    }

    Ok(achieved_achievements)
}

pub async fn get_user(
    Path((id, slug)): Path<(i32, String)>,
    account: Option<Account>,
    Extension(pool): Extension<PgPool>,
) -> Result<UserInfo, Error> {
    let account_info = AccountProfileInfo::fetch(&pool, id)
        .await
        .map_err(Error::Database)?;
    let Some(account_info) = account_info else {
        return Err(Error::NotFound);
    };

    if format!("{}", Slug(&account_info.username)) != slug {
        return Err(Error::Redirect(Cow::Owned(format!(
            "/user/{id}/{}",
            Slug(&account_info.username)
        ))));
    }

    let invalidated_solutions = match account {
        Some(ref acc) if acc.id == id => Some(
            InvalidatedSolution::get_invalidated_solutions_for_user(id, &pool)
                .await
                .map_err(Error::Database)?,
        ),
        _ => None,
    };

    let solutions = query_as!(
        UserPageLeaderboardEntry,
        "SELECT solutions.language, solutions.points, solutions.challenge as challenge_id, challenges.name as challenge_name
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
        recent_achievements: get_user_achievements(&pool, id, account.is_some_and(|i| i.id == id))
            .await
            .map_err(Error::Database)?,
        per_language_stats: get_account_language_stats(&pool, id)
            .await
            .map_err(Error::Database)?,
        per_category_stats: get_account_category_stats(&pool, id)
            .await
            .map_err(Error::Database)?,
    })
}
