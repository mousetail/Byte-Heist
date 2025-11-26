use serde::{Deserialize, Serialize};
use sqlx::{PgPool, query_as, query_scalar};
use tower_sessions::cookie::time::OffsetDateTime;

use super::GetById;

pub struct SolutionWithLanguage {
    pub points: i32,
    pub is_post_mortem: bool,
    pub language: String,
    pub author: i32,
    pub author_name: String,
}

impl SolutionWithLanguage {
    pub async fn get_best_per_language(
        pool: &PgPool,
        challenge_id: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            SolutionWithLanguage,
            r#"
                SELECT DISTINCT ON (language)
                    points,
                    is_post_mortem,
                    language,
                    author,
                    accounts.username as author_name
                FROM solutions
                LEFT JOIN accounts ON solutions.author = accounts.id
                WHERE valid AND not is_post_mortem AND challenge=$1
                ORDER BY language ASC, points ASC, solutions.created_at ASC
            "#,
            challenge_id
        )
        .fetch_all(pool)
        .await
    }
}

impl GetById for SolutionWithLanguage {
    async fn get_by_id(pool: &PgPool, id: i32) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            SolutionWithLanguage,
            r#"
                SELECT
                    points,
                    is_post_mortem,
                    language,
                    author,
                    accounts.username as author_name
                FROM solutions
                LEFT JOIN accounts on solutions.author = accounts.id
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
    pub points: i32,
    pub id: i32,
    pub valid: bool,
    pub last_improved_date: OffsetDateTime,
    pub is_post_mortem: bool,
    pub score: Option<i32>,
    pub rank: Option<i64>,
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
                    solutions.points,
                    solutions.id,
                    valid,
                    last_improved_date,
                    is_post_mortem as "is_post_mortem!",
                    score,
                    rank
                FROM solutions
                LEFT JOIN scores
                ON scores.id = solutions.id
                WHERE solutions.author=$1 AND solutions.challenge=$2 AND solutions.language=$3
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
    pub points: i32,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum RankingMode {
    #[default]
    Top,
    Me,
}

#[derive(Serialize)]
pub struct ScoreInfo {
    pub rank: usize,
    pub points: i32,
    pub score: usize,
}

pub struct LeaderboardNearOutput {
    pub leaderboard: Vec<LeaderboardEntry>,
    pub score_info: Option<ScoreInfo>,
}

impl LeaderboardEntry {
    fn calculate_user_score(rank: usize, leaderboard: &[LeaderboardEntry]) -> i32 {
        let percentile_50th = if leaderboard.len() > 2 {
            leaderboard[leaderboard.len() / 2].points
        } else {
            9999
        };
        let percentile_90th = if leaderboard.len() > 1 {
            leaderboard[leaderboard.len() * 9 / 10].points
        } else {
            9999
        };
        let percentile_10th = if leaderboard.len() > 9 {
            leaderboard[leaderboard.len() / 10].points
        } else {
            9999
        };
        let points = leaderboard[rank - 1].points;

        (if rank == 1 { 10 } else { 0 })
            + (percentile_90th.saturating_sub(points).max(0) / 4).min(50)
            + (percentile_50th.saturating_sub(points).max(0) / 2).min(50)
            + percentile_10th.saturating_sub(points).clamp(0, 49)
            + 1
    }

    fn truncate_leaderboard(
        mut leaderboard: Vec<LeaderboardEntry>,
        mode: RankingMode,
        user_id: Option<i32>,
    ) -> Vec<LeaderboardEntry> {
        let user_rank =
            user_id.and_then(|user_id| leaderboard.iter().position(|e| e.author_id == user_id));
        match mode {
            RankingMode::Top => {
                leaderboard.truncate(10);
                leaderboard
            }
            RankingMode::Me => {
                let index = user_rank.unwrap_or(0);
                let mut start = index.saturating_sub(5);
                let mut end = start + 10;
                if end >= leaderboard.len() {
                    let diff = start.min(end - leaderboard.len());
                    start -= diff;
                    end = (end - diff).min(leaderboard.len());
                }
                leaderboard[start..end].to_vec()
            }
        }
    }

    pub async fn get_leaderboard_near(
        pool: &PgPool,
        challenge_id: i32,
        language: &str,
        user_id: Option<i32>,
        mode: RankingMode,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let leaderboard =
            Self::get_leadeboard_for_challenge_and_language(pool, challenge_id, language).await?;

        Ok(Self::truncate_leaderboard(leaderboard, mode, user_id))
    }

    pub async fn get_leaderboard_and_scores_near(
        pool: &PgPool,
        challenge_id: i32,
        language: &str,
        user_id: Option<i32>,
        mode: RankingMode,
    ) -> Result<LeaderboardNearOutput, sqlx::Error> {
        let leaderboard =
            Self::get_leadeboard_for_challenge_and_language(pool, challenge_id, language).await?;
        let user_rank = user_id
            .and_then(|user_id| leaderboard.iter().position(|e| e.author_id == user_id))
            .map(|i| i + 1);
        let user_score =
            user_rank.map(|user_rank| Self::calculate_user_score(user_rank, &leaderboard));

        let score_info = user_rank.zip(user_score).map(|(rank, score)| ScoreInfo {
            rank,
            points: leaderboard[rank - 1].points as i32,
            score: score as usize,
        });

        Ok(LeaderboardNearOutput {
            leaderboard: Self::truncate_leaderboard(leaderboard, mode, user_id),
            score_info,
        })
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
                points
            FROM solutions
                LEFT JOIN accounts ON solutions.author = accounts.id
            WHERE solutions.challenge=$1 AND solutions.language=$2 AND valid=true
            ORDER BY solutions.points ASC, last_improved_date ASC
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
                points,
                rank() OVER (ORDER BY solutions.points ASC) as "rank!"
            FROM solutions
                LEFT JOIN accounts ON solutions.author = accounts.id
            WHERE solutions.challenge=$1 AND solutions.language=$2 AND valid=true
            ORDER BY solutions.points ASC, last_improved_date ASC
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
