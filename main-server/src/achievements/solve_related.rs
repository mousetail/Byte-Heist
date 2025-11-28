use sqlx::{PgPool, query};

use crate::achievements::AchievementType;

pub(super) async fn award_solve_related(pool: &PgPool) -> Result<(), sqlx::Error> {
    let first_day_solve_name: &str = AchievementType::FirstDaySolve.into();
    let last_day_solve_name: &str = AchievementType::LastDaySolve.into();

    query!(
        "INSERT INTO achievements(user_id, achievement, related_language, related_challenge, awarded_at, achieved)
        SELECT
            account as user_id,
            $1,
            language as related_language,
            challenge as related_challenge,
            user_activities.date,
            true
        FROM user_activities
        LEFT JOIN
            challenges ON user_activities.challenge=challenges.id
        WHERE user_activities.date > now() + interval '-8 days'
            AND user_activities.date >= challenges.go_live_date::date
            AND user_activities.date <= challenges.go_live_date::date + interval '+1 day'
        ON CONFLICT DO NOTHING
        ",
        first_day_solve_name
    )
    .execute(pool)
    .await?;

    query!(
        "INSERT INTO achievements(user_id, achievement, related_language, related_challenge, awarded_at, achieved)
        SELECT
            account as user_id,
            $1,
            language as related_language,
            challenge as related_challenge,
            user_activities.date,
            true
        FROM user_activities
        LEFT JOIN
            challenges ON user_activities.challenge=challenges.id
        WHERE user_activities.date > now() + interval '-8 days'
            AND user_activities.date > challenges.post_mortem_date::date + interval '-1 day'
            AND user_activities.date < challenges.post_mortem_date::date
        ON CONFLICT DO NOTHING
        ",
        last_day_solve_name
    )
    .execute(pool)
    .await?;

    Ok(())
}
