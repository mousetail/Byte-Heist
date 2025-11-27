use sqlx::{PgPool, query};

use crate::models::challenge::ChallengeCategory;

use super::AchievementType;

async fn point_based_score(
    pool: &PgPool,
    achievement_type: AchievementType,
    category: ChallengeCategory,
    minimum_score: i64,
) -> Result<(), sqlx::Error> {
    let achievement_type_string: &'static str = achievement_type.into();
    query!(
        r#"
        INSERT INTO achievements(
            user_id,
            achievement,
            achieved,
            awarded_at
        ) SELECT
            author as user_id,
            $1,
            true,
            now()
        FROM user_scoring_info
        WHERE total_score > $2 AND category = $3::challenge_category
        ON CONFLICT DO NOTHING
    "#,
        achievement_type_string,
        minimum_score,
        category as ChallengeCategory
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn point_based_score_for_lang(
    pool: &PgPool,
    achievement_type: AchievementType,
    minimum_score: i64,
    language: &'static str,
) -> Result<(), sqlx::Error> {
    let achievement_type_string: &'static str = achievement_type.into();
    query!(
        r#"
        INSERT INTO achievements(
            user_id,
            achievement,
            achieved,
            awarded_at
        ) SELECT
            author as user_id,
            $1,
            true,
            now()
        FROM user_scoring_info_per_language
        WHERE language=$2
        GROUP BY author
        HAVING CAST(SUM(total_score) as BIGINT) > $3
        ON CONFLICT DO NOTHING
    "#,
        achievement_type_string,
        language,
        minimum_score.into(),
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub(super) async fn award_point_based_cheevos(pool: &PgPool) -> Result<(), sqlx::Error> {
    point_based_score(
        pool,
        AchievementType::CodeGolf1Point,
        ChallengeCategory::CodeGolf,
        1,
    )
    .await?;
    point_based_score(
        pool,
        AchievementType::CodeGolf250Point,
        ChallengeCategory::CodeGolf,
        250,
    )
    .await?;
    point_based_score(
        pool,
        AchievementType::CodeGolf500Point,
        ChallengeCategory::CodeGolf,
        500,
    )
    .await?;
    point_based_score(
        pool,
        AchievementType::CodeGolf1000Point,
        ChallengeCategory::CodeGolf,
        1000,
    )
    .await?;
    point_based_score(
        pool,
        AchievementType::CodeGolf2000Point,
        ChallengeCategory::CodeGolf,
        2000,
    )
    .await?;

    point_based_score(
        pool,
        AchievementType::RestrictedSource1Point,
        ChallengeCategory::RestrictedSource,
        1,
    )
    .await?;
    point_based_score(
        pool,
        AchievementType::RestrictedSource250Point,
        ChallengeCategory::RestrictedSource,
        250,
    )
    .await?;
    point_based_score(
        pool,
        AchievementType::RestrictedSource500Point,
        ChallengeCategory::RestrictedSource,
        500,
    )
    .await?;
    point_based_score(
        pool,
        AchievementType::RestrictedSource1000Point,
        ChallengeCategory::RestrictedSource,
        1000,
    )
    .await?;
    point_based_score(
        pool,
        AchievementType::RestrictedSource2000Point,
        ChallengeCategory::RestrictedSource,
        2000,
    )
    .await?;

    point_based_score_for_lang(pool, AchievementType::Python1000Point, 1000, "python").await?;
    point_based_score_for_lang(pool, AchievementType::JavaScript1000Point, 1000, "nodejs").await?;
    point_based_score_for_lang(pool, AchievementType::C1000Point, 1000, "tcc").await?;
    point_based_score_for_lang(pool, AchievementType::Rust1000Point, 1000, "rust").await?;
    point_based_score_for_lang(pool, AchievementType::Vyxal1000Point, 1000, "vyxal3").await?;
    point_based_score_for_lang(pool, AchievementType::Apl1000Point, 1000, "apl").await?;

    Ok(())
}
