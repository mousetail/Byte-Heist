use common::AchievementType;
use sqlx::{PgPool, query};

pub(super) async fn award_solve_related(pool: &PgPool) -> Result<(), sqlx::Error> {
    let first_day_solve_name: &str = AchievementType::FirstDaySolve.into();
    let last_day_solve_name: &str = AchievementType::LastDaySolve.into();
    let ten_top_ten_name: &str = AchievementType::TenTopTen.into();
    let one_point_name: &str = AchievementType::OnePoint.into();
    let solve_impossible: &str = AchievementType::SolveImpossible.into();

    query!(
        r#"
            INSERT INTO achievements(
                user_id,
                achievement,
                awarded_at,
                achieved,
                progress,
                total
            ) SELECT
                author as user_id,
                $1,
                case when SUM(top_ten_percents) >= 10 then now() else null end,
                SUM(top_ten_percents) >=10,
                LEAST(10, SUM(top_ten_percents)),
                10
            FROM user_scoring_info
            GROUP BY author
            HAVING SUM(top_ten_percents) > 0
            ON CONFLICT(user_id, achievement) DO UPDATE SET
                achieved = achievements.achieved OR excluded.achieved,
                progress=case when achievements.achieved then 10 else excluded.progress end,
                awarded_at = coalesce(achievements.awarded_at, excluded.awarded_at)

        "#,
        ten_top_ten_name
    )
    .execute(pool)
    .await?;

    query!(
        r#"
            INSERT INTO achievements(
                user_id,
                achievement,
                awarded_at,
                achieved,
                related_challenge,
                related_language
            ) SELECT DISTINCT ON (author)
                author as user_id,
                $1,
                now(),
                true,
                challenge,
                language
            FROM scores
            WHERE score = 1
            ON CONFLICT DO NOTHING
        "#,
        one_point_name
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

    query!(
        "INSERT INTO achievements(user_id, achievement, related_language, related_challenge, awarded_at, achieved)
        SELECT
            author as user_id,
            $1,
            language as related_language,
            challenge as related_challenge,
            created_at,
            true
        FROM solutions
        WHERE challenge=17
        ON CONFLICT DO NOTHING
        ",
        solve_impossible
    )
    .execute(pool)
    .await?;

    Ok(())
}
