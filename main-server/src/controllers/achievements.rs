use std::str::FromStr;

use axum::{Extension, extract::Path};
use common::AchievementType;
use serde::Serialize;
use sqlx::{PgPool, query_as};
use strum::VariantArray;
use time::OffsetDateTime;

use crate::error::Error;

#[derive(Serialize)]
pub struct Achievement {
    icon: String,
}

pub async fn list_achievements(_path: Option<Path<String>>) -> Result<Vec<Achievement>, Error> {
    Ok(AchievementType::VARIANTS
        .iter()
        .map(|i| Achievement { icon: i.get_icon() })
        .collect())
}

#[derive(Serialize)]
struct UserAchievement {
    user_id: Option<i32>,
    user_username: String,
    user_avatar: String,
    awarded_at: OffsetDateTime,
    related_challenge_id: Option<i32>,
    related_challenge_name: Option<String>,
    related_language: Option<String>,
}

impl UserAchievement {
    async fn get_all(
        pool: &PgPool,
        achievement_type: AchievementType,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let achievement_type_string: &str = achievement_type.into();
        query_as!(
            Self,
            r#"
                SELECT
                    user_id as "user_id?",
                    related_challenge as related_challenge_id,
                    related_language as "related_language?",
                    accounts.username as "user_username",
                    accounts.avatar as "user_avatar",
                    challenges.name as "related_challenge_name?",
                    awarded_at as "awarded_at!"
                FROM achievements
                INNER JOIN
                    accounts on accounts.id = user_id
                LEFT JOIN
                    challenges on challenges.id = achievements.related_challenge
                WHERE
                    awarded_at IS NOT NULL
                    AND achieved
                AND achievement=$1
                ORDER BY awarded_at DESC
            "#,
            achievement_type_string
        )
        .fetch_all(pool)
        .await
    }
}

#[derive(Serialize)]
pub struct GetAchievementOutput {
    achievements: Vec<UserAchievement>,
    achievement_name: String,
}

pub async fn get_achievement(
    Extension(pool): Extension<PgPool>,
    Path(path): Path<String>,
) -> Result<GetAchievementOutput, Error> {
    let achievement_type: AchievementType =
        AchievementType::from_str(&path).map_err(|_| Error::NotFound)?;

    let achievements = UserAchievement::get_all(&pool, achievement_type)
        .await
        .map_err(Error::Database)?;

    Ok(GetAchievementOutput {
        achievements,
        achievement_name: path,
    })
}
