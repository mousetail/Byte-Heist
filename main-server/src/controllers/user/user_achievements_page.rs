use std::{borrow::Cow, collections::HashMap, str::FromStr};

use axum::{Extension, extract::Path};
use serde::Serialize;
use sqlx::{PgPool, query_as};
use strum::VariantArray;
use time::OffsetDateTime;

use crate::{
    achievements::{AchievementCategory, AchievementType},
    error::Error,
};

#[derive(Serialize, Debug)]
struct UserPageAchievementInfo {
    achievement: Cow<'static, str>,
    progress: Option<i64>,
    total: Option<i64>,
    awarded_at: Option<OffsetDateTime>,
}

async fn get_all_achievements_for_user(
    pool: &PgPool,
    user_id: i32,
) -> Result<Vec<UserPageAchievementInfo>, sqlx::Error> {
    query_as!(
        UserPageAchievementInfo,
        r#"
            SELECT achievement, progress, total, awarded_at
            FROM achievements
            WHERE user_id=$1
        
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
}

#[derive(Serialize)]
pub struct AchievementGroup {
    group: AchievementCategory,
    elements: Vec<UserPageAchievementInfo>,
}

fn categorize_achievements(achievements: Vec<UserPageAchievementInfo>) -> Vec<AchievementGroup> {
    let mut map: HashMap<AchievementType, UserPageAchievementInfo> = achievements
        .into_iter()
        .map(|i| Some((AchievementType::from_str(&i.achievement).ok()?, i)))
        .collect::<Option<_>>()
        .expect("At least one invalid achievement type in the database");

    println!("{map:?}");

    let mut categories = HashMap::new();
    for achievement_type in AchievementType::VARIANTS {
        categories
            .entry(achievement_type.get_achievement_category())
            .or_insert_with(|| vec![])
            .push(
                map.remove(achievement_type)
                    .unwrap_or_else(|| UserPageAchievementInfo {
                        achievement: Cow::Borrowed(achievement_type.into()),
                        progress: None,
                        total: None,
                        awarded_at: None,
                    }),
            );
    }

    categories
        .into_iter()
        .map(|(group, items)| AchievementGroup {
            group,
            elements: items,
        })
        .collect()
}

pub async fn get_user_achievements(
    Path((user_id, _slug)): Path<(i32, String)>,
    Extension(pool): Extension<PgPool>,
) -> Result<Vec<AchievementGroup>, Error> {
    let achievements = get_all_achievements_for_user(&pool, user_id)
        .await
        .map_err(Error::Database)?;
    let grouped_achievements = categorize_achievements(achievements);

    Ok(grouped_achievements)
}
