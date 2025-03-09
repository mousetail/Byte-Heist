use serde::{Deserialize, Serialize};
use sqlx::{query_as, query_scalar, PgPool};
use tower_sessions::cookie::time::OffsetDateTime;

use super::GetById;

pub struct SolutionWithLanguage {
    pub score: i32,
    pub is_post_mortem: bool,
    pub language: String,
    pub author: i32,
}

impl GetById for SolutionWithLanguage {
    async fn get_by_id(pool: &PgPool, id: i32) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            SolutionWithLanguage,
            r#"
                SELECT
                    score,
                    is_post_mortem as "is_post_mortem!",
                    language,
                    author
                FROM solutions
                WHERE solutions.id=$1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
    }
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct NewSolution {
    pub code: String,
}

#[derive(Serialize)]
pub struct Code {
    pub code: String,
    pub score: i32,
    pub id: i32,
    pub valid: bool,
    pub last_improved_date: OffsetDateTime,
    pub is_post_mortem: bool,
}

impl Code {
    pub async fn get_best_code_for_user(
        pool: &PgPool,
        account: i32,
        challenge: i32,
        language: &str,
    ) -> Option<Code> {
        sqlx::query_as!(
            Code,
            r#"
                SELECT
                    code, 
                    score,
                    id,
                    valid,
                    last_improved_date,
                    is_post_mortem as "is_post_mortem!" from solutions
                WHERE author=$1 AND challenge=$2 AND language=$3
                ORDER BY is_post_mortem DESC, score ASC
                LIMIT 1
            "#,
            account,
            challenge,
            language
        )
        .fetch_optional(pool)
        .await
        .expect("Database connection error")
    }
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Debug, Clone)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub id: i32,
    pub author_id: i32,
    pub author_name: String,
    pub author_avatar: String,
    pub score: i32,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum RankingMode {
    #[default]
    Top,
    Me,
}

impl LeaderboardEntry {
    pub async fn get_leaderboard_near(
        pool: &PgPool,
        challenge_id: i32,
        language: &str,
        user_id: Option<i32>,
        mode: RankingMode,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let mut leaderboard =
            Self::get_leadeboard_for_challenge_and_language(pool, challenge_id, language).await?;

        match mode {
            RankingMode::Top => {
                leaderboard.truncate(10);
                Ok(leaderboard)
            }
            RankingMode::Me => {
                let index = leaderboard
                    .iter()
                    .position(|k| Some(k.author_id) == user_id)
                    .unwrap_or(0);
                let mut start = index.saturating_sub(5);
                let mut end = start + 10;
                if end >= leaderboard.len() {
                    let diff = start.min(end - leaderboard.len());
                    start -= diff;
                    end = (end - diff).min(leaderboard.len());
                }
                Ok(leaderboard[start..end].to_vec())
            }
        }
    }

    pub async fn get_top_entry(
        pool: &PgPool,
        challenge_id: i32,
        language: &str,
    ) -> Result<Option<LeaderboardEntry>, sqlx::Error> {
        sqlx::query_as!(
            LeaderboardEntry,
            r#"
            SELECT
                solutions.id as id,
                solutions.author as author_id,
                accounts.username as author_name,
                accounts.avatar as author_avatar,
                1 as "rank!",
                score
            FROM solutions
                LEFT JOIN accounts ON solutions.author = accounts.id
            WHERE solutions.challenge=$1 AND solutions.language=$2 AND valid=true
            ORDER BY solutions.score ASC, last_improved_date ASC
            LIMIT 1
            "#,
            challenge_id,
            language
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn get_leadeboard_for_challenge_and_language(
        pool: &PgPool,
        challenge_id: i32,
        language: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            LeaderboardEntry,
            r#"
            SELECT
                solutions.id as id,
                solutions.author as author_id,
                accounts.username as author_name,
                accounts.avatar as author_avatar,
                score,
                rank() OVER (ORDER BY solutions.score ASC) as "rank!"
            FROM solutions
                LEFT JOIN accounts ON solutions.author = accounts.id
            WHERE solutions.challenge=$1 AND solutions.language=$2 AND valid=true
            ORDER BY solutions.score ASC, last_improved_date ASC
            "#,
            challenge_id,
            language
        )
        .fetch_all(pool)
        .await
    }
}

#[derive(Serialize)]
pub struct InvalidatedSolution {
    language: String,
    challenge_id: i32,
    challenge_name: String,
}

impl InvalidatedSolution {
    pub async fn get_invalidated_solutions_for_user(
        user: i32,
        pool: &PgPool,
    ) -> Result<Vec<InvalidatedSolution>, sqlx::Error> {
        let result = query_as!(
            InvalidatedSolution,
            "SELECT solutions.language, challenges.id as challenge_id, challenges.name as challenge_name
            FROM solutions LEFT JOIN challenges ON solutions.challenge = challenges.id
            WHERE solutions.valid = false AND solutions.author = $1",
            user
        ).fetch_all(pool).await?;

        Ok(result)
    }

    pub async fn invalidated_solution_exists(
        user: i32,
        pool: &PgPool,
    ) -> Result<bool, sqlx::Error> {
        Ok(query_scalar!(
            "SELECT EXISTS (SELECT * FROM solutions WHERE valid=false AND author=$1)",
            user
        )
        .fetch_one(pool)
        .await?
        .unwrap())
    }
}
