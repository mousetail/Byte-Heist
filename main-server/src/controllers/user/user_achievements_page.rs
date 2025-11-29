use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    str::FromStr,
};

use axum::{Extension, extract::Path};
use common::slug::Slug;
use serde::Serialize;
use sqlx::{PgPool, query_as};
use strum::VariantArray;
use time::OffsetDateTime;

use crate::{
    achievements::{AchievementCategory, AchievementType},
    error::Error,
};

use super::user_main_page::AccountProfileInfo;

#[derive(Serialize, Debug)]
struct UserPageAchievementInfo {
    achievement: Cow<'static, str>,
    progress: Option<i64>,
    total: Option<i64>,
    awarded_at: Option<OffsetDateTime>,
    total_awarded: i64,
}

async fn get_all_achievements_for_user(
    pool: &PgPool,
    user_id: i32,
) -> Result<Vec<UserPageAchievementInfo>, sqlx::Error> {
    query_as!(
        UserPageAchievementInfo,
        r#"
            WITH known_achievement_types as (
                SELECT achievements.achievement
                FROM achievements
                WHERE user_id=$1
                UNION DISTINCT SELECT achievement_stats.achievement
                FROM achievement_stats)
            SELECT DISTINCT ON ("achievement!")
                known_achievement_types.achievement as "achievement!",
                achievements.progress,
                achievements.total,
                achievements.awarded_at,
                COALESCE(achievement_stats.total_awarded, 0) as "total_awarded!"
            FROM known_achievement_types
            LEFT JOIN achievements
            ON achievements.achievement = known_achievement_types.achievement AND achievements.user_id=$1
            LEFT JOIN achievement_stats
            ON achievement_stats.achievement = known_achievement_types.achievement
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
    earned: usize,
}

fn categorize_achievements(achievements: Vec<UserPageAchievementInfo>) -> Vec<AchievementGroup> {
    let mut map: HashMap<AchievementType, UserPageAchievementInfo> = achievements
        .into_iter()
        .map(|i| Some((AchievementType::from_str(&i.achievement).ok()?, i)))
        .collect::<Option<_>>()
        .expect("At least one invalid achievement type in the database");

    let mut categories = BTreeMap::new();
    for achievement_type in AchievementType::VARIANTS {
        categories
            .entry(achievement_type.get_achievement_category())
            .or_insert_with(std::vec::Vec::new)
            .push(
                map.remove(achievement_type)
                    .unwrap_or_else(|| UserPageAchievementInfo {
                        achievement: Cow::Borrowed(achievement_type.into()),
                        progress: None,
                        total: None,
                        awarded_at: None,
                        total_awarded: 0,
                    }),
            );
    }

    categories
        .into_iter()
        .map(|(group, items)| AchievementGroup {
            group,
            earned: items.iter().filter(|i| i.awarded_at.is_some()).count(),
            elements: items,
        })
        .collect()
}

#[derive(Serialize)]
pub struct UserAchievementPageData {
    account_info: AccountProfileInfo,
    achievement_groups: Vec<AchievementGroup>,
}

pub async fn get_user_achievements(
    Path((user_id, slug)): Path<(i32, String)>,
    Extension(pool): Extension<PgPool>,
) -> Result<UserAchievementPageData, Error> {
    let achievements = get_all_achievements_for_user(&pool, user_id)
        .await
        .map_err(Error::Database)?;

    let account_info = AccountProfileInfo::fetch(&pool, user_id)
        .await
        .map_err(Error::Database)?;
    let Some(account_info) = account_info else {
        return Err(Error::NotFound);
    };

    if format!("{}", Slug(&account_info.username)) != slug {
        return Err(Error::Redirect(Cow::Owned(format!(
            "/user/{user_id}/{}",
            Slug(&account_info.username)
        ))));
    }

    let grouped_achievements = categorize_achievements(achievements);

    Ok(UserAchievementPageData {
        account_info,
        achievement_groups: grouped_achievements,
    })
}
