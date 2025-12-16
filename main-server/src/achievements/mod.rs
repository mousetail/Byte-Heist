mod misc;
mod points_based;
mod solve_related;
pub mod vote_achievements;

use common::AchievementType;
use misc::award_misc_achievements;
use points_based::award_point_based_cheevos;
use solve_related::award_solve_related;
use sqlx::{PgPool, query, query_scalar};

pub async fn award_achievements(pool: &PgPool) -> Result<(), sqlx::Error> {
    award_point_based_cheevos(pool).await?;
    award_misc_achievements(pool)
        .await
        .inspect_err(|e| eprintln!("Failed to award github achievement: {e:?}"))
        .unwrap();
    award_solve_related(pool).await?;
    Ok(())
}

pub async fn get_unread_achievements_for_user(
    pool: &PgPool,
    user_id: i32,
) -> Result<Vec<String>, sqlx::Error> {
    query_scalar!(
        "UPDATE achievements
        SET read=true
        WHERE user_id=$1 AND read=false AND achieved=true
        RETURNING achievement",
        user_id
    )
    .fetch_all(pool)
    .await
}

pub async fn award_achievement(
    pool: &PgPool,
    user: i32,
    achievement_type: AchievementType,
    associated_challenge: Option<i32>,
    associated_language: Option<&str>,
) -> Result<(), sqlx::Error> {
    let achievement_name: &str = achievement_type.into();
    query!(
        r#"INSERT INTO achievements(user_id, achievement, related_challenge, related_language, awarded_at, achieved)
        VALUES ($1, $2, $3, $4, now(), true)
        ON CONFLICT DO NOTHING
        "#,
        user,
        achievement_name,
        associated_challenge,
        associated_language
    ).execute(pool).await?;

    Ok(())
}
