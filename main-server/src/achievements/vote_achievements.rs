use common::AchievementType;
use sqlx::{PgPool, query};

pub async fn award_vote_achievements(pool: &PgPool, account_id: i32) -> Result<(), sqlx::Error> {
    for achievement in [
        AchievementType::AcceptedVote5,
        AchievementType::AcceptedVote25,
        AchievementType::AcceptedVote125,
    ] {
        let achievement_type_name: &str = achievement.into();
        query!(
            r#"
                INSERT INTO achievements(user_id, achievement, achieved, awarded_at, progress, total)
                VALUES ($1, $2, false, null, 1, $3)
                ON CONFLICT(user_id, achievement) DO UPDATE SET
                    achieved=achievements.progress + 1 >= $3,
                    progress = LEAST(achievements.progress + 1, $3),
                    awarded_at=COALESCE(achievements.awarded_at, case when achievements.progress + 1 >= $3 then now() else null end)
            "#,
            account_id,
            achievement_type_name,
            achievement
                .get_associated_number()
                .expect("Expected voting related achievements to have an associated number") as i64
        ).execute(pool).await?;
    }

    Ok(())
}
