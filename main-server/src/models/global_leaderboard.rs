use serde::Serialize;
use sqlx::{query_as, types::time::OffsetDateTime, PgPool};

use crate::error::Error;

use super::challenge::ChallengeCategory;

#[derive(Serialize)]
pub struct GlobalLeaderboardEntry {
    author_name: String,
    author_id: i32,
    author_avatar: String,
    author_join_date: OffsetDateTime,
    first_places: i64,
    total_score: i64,
    solutions: i64,
    rank: i64,
}

impl GlobalLeaderboardEntry {
    pub async fn get_all_by_language(
        pool: &PgPool,
        category: ChallengeCategory,
        language: &str,
    ) -> Result<Vec<Self>, Error> {
        query_as!(
            GlobalLeaderboardEntry,
            r#"
                SELECT
                    user_scoring_info.author as "author_id!",
                    accounts.username as author_name,
                    accounts.avatar as author_avatar,
                    accounts.created_at as author_join_date,

                    user_scoring_info.first_places AS "first_places!",
                    CAST(user_scoring_info.sols AS bigint) as "solutions!:i64",
                    user_scoring_info.total_score as "total_score!:i64",
                    user_scoring_info.rank as "rank!:i64"
                FROM user_scoring_info_per_language as user_scoring_info
                INNER JOIN accounts
                    ON user_scoring_info.author = accounts.id
                WHERE
                    user_scoring_info.category = $1
                    AND user_scoring_info.language = $2
                ORDER BY "total_score!:i64" DESC
            "#,
            category as ChallengeCategory,
            language
        )
        .fetch_all(pool)
        .await
        .map_err(Error::Database)
    }

    pub async fn get_all(pool: &PgPool, category: ChallengeCategory) -> Result<Vec<Self>, Error> {
        query_as!(
            GlobalLeaderboardEntry,
            r#"
                SELECT
                    user_scoring_info.author as "author_id!",
                    accounts.username as author_name,
                    accounts.avatar as author_avatar,
                    accounts.created_at as author_join_date,

                    user_scoring_info.first_places AS "first_places!",
                    CAST(user_scoring_info.sols AS bigint) as "solutions!:i64",
                    user_scoring_info.total_score as "total_score!:i64",
                    user_scoring_info.rank as "rank!:i64"
                FROM user_scoring_info
                INNER JOIN accounts
                    ON user_scoring_info.author = accounts.id
                WHERE
                    user_scoring_info.category = $1
                ORDER BY "total_score!:i64" DESC
            "#,
            category as ChallengeCategory
        )
        .fetch_all(pool)
        .await
        .map_err(Error::Database)
    }
}
