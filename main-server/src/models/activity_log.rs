use sqlx::PgPool;
use sqlx::query;

pub async fn save_activity_log(
    pool: &PgPool,
    challenge_id: i32,
    user_id: i32,
    language: &str,
    old_score: Option<i32>,
    new_score: i32,
) -> Result<(), sqlx::Error> {
    query!(
        r#"
            INSERT INTO user_activities(account, challenge, old_points, new_points, language)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (account, challenge, date, language) DO UPDATE SET new_points=$4
        "#,
        user_id,
        challenge_id,
        old_score,
        new_score,
        language
    )
    .execute(pool)
    .await?;

    Ok(())
}
