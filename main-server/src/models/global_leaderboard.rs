use serde::Serialize;
use sqlx::{query_as, PgPool};

use crate::error::Error;

use super::challenge::ChallengeCategory;

#[derive(Serialize)]
pub struct GlobalLeaderboardEntry {
    author_name: String,
    author_id: i32,
    author_avatar: String,
    total_score: i32,
    solutions: i64,
    rank: i64,
}

impl GlobalLeaderboardEntry {
    pub async fn get_all(pool: &PgPool, category: ChallengeCategory) -> Result<Vec<Self>, Error> {
        query_as!(
            GlobalLeaderboardEntry,
            r#"
                SELECT
                    scores.author as "author_id!",
                    accounts.username as author_name,
                    accounts.avatar as author_avatar,
                    COUNT(*) as "solutions!",
                    CAST(SUM(scores.score) AS integer) as "total_score!:i32",
                    rank() OVER (ORDER BY CAST(SUM(scores.score) AS integer) DESC) as "rank!"
                FROM scores
                INNER JOIN accounts
                    ON scores.author = accounts.id
                INNER JOIN challenges
                    ON scores.challenge = challenges.id
                WHERE
                    challenges.category = $1
                    AND challenges.status = 'public'
                GROUP BY scores.author, accounts.username, accounts.avatar
                ORDER BY "total_score!:i32" DESC
            "#,
            category as ChallengeCategory
        )
        .fetch_all(pool)
        .await
        .map_err(Error::Database)
    }
}
