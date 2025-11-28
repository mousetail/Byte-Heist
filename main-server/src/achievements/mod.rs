mod misc;
mod points_based;

use std::hash::{DefaultHasher, Hash, Hasher};

use misc::award_misc_achievements;
use points_based::award_point_based_cheevos;
use serde::Serialize;
use sqlx::{PgPool, query_scalar};
use strum::{EnumString, IntoStaticStr, VariantArray};

#[derive(Serialize, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum AchievementCategory {
    PointRelated,
    Miscellaneous,
    LanguageRelated,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, VariantArray, IntoStaticStr, EnumString, Debug)]
pub enum AchievementType {
    // Solve Related
    // OnePoint,
    // FirstPlace,
    // OnlySolution,
    // FiveLanguages,
    // ImproveFirstPlace,
    // FirstDaySolve,
    // LastDaySolve,
    // Change Suggestion Related
    // ChangeSuggestionInvalidates3,
    // ChangeSuggestionInvalidate12,
    // UpvoteSuggestionThatInvalidates3,
    // ImproveDescription,
    // ImproveJudge,
    // Creating Challenge Related
    // SubmitCodeGolf,
    // SubmitRestrictedSource,
    // HeistGoLive,
    // ContributeToLiveHeist,
    // SubmitABetaHeist,
    // TopThreeOnLiveDate,
    // InvalidOnLiveDate,
    // Points Related
    CodeGolf1Point,
    CodeGolf250Point,
    CodeGolf500Point,
    CodeGolf1000Point,
    CodeGolf2000Point,
    RestrictedSource1Point,
    RestrictedSource250Point,
    RestrictedSource500Point,
    RestrictedSource1000Point,
    RestrictedSource2000Point,
    // Language Related
    Python1000Point,
    JavaScript1000Point,
    Rust1000Point,
    Vyxal1000Point,
    C1000Point,
    Apl1000Point,
    // Site Features
    // ReadDocumentation,
    // SubmitAPullRequest,
    // Miscellaneous
    // SolveImpossible,
    StarTheRepo,
}

impl AchievementType {
    pub fn get_achievement_name(self) -> &'static str {
        match self {
            AchievementType::CodeGolf1Point => "Code Golf Baby",
            AchievementType::CodeGolf250Point => "Code Golf Newbie",
            AchievementType::CodeGolf500Point => "Code Golf Starter",
            AchievementType::CodeGolf1000Point => "Code Golf Junior",
            AchievementType::CodeGolf2000Point => "Code Golf Intermediate",
            AchievementType::RestrictedSource1Point => "Restricted Source Baby",
            AchievementType::RestrictedSource250Point => "Restricted Source Newbie",
            AchievementType::RestrictedSource500Point => "Restricted Source Starter",
            AchievementType::RestrictedSource1000Point => "Restricted Source Junior",
            AchievementType::RestrictedSource2000Point => "Restricted Source Intermediate",
            AchievementType::Python1000Point => "Snake Charmer",
            AchievementType::JavaScript1000Point => "[Insert Joke Here]",
            AchievementType::Rust1000Point => "[Insert Joke Here]",
            AchievementType::Vyxal1000Point => "[Insert Joke Here]",
            AchievementType::C1000Point => "Deep Blue Sea",
            AchievementType::Apl1000Point => "Original Sin",
            AchievementType::StarTheRepo => "Starstruck",
        }
    }

    pub fn get_achievement_category(self) -> AchievementCategory {
        match self {
            AchievementType::StarTheRepo => AchievementCategory::Miscellaneous,
            AchievementType::Python1000Point
            | AchievementType::JavaScript1000Point
            | AchievementType::Apl1000Point
            | AchievementType::C1000Point
            | AchievementType::Rust1000Point
            | AchievementType::Vyxal1000Point => AchievementCategory::LanguageRelated,
            _ => AchievementCategory::PointRelated,
        }
    }

    pub fn get_achievement_description(self) -> &'static str {
        match self {
            AchievementType::CodeGolf1Point => "Earn your first point in Code Golf",
            AchievementType::CodeGolf250Point => "Earn 250 points in Code Golf",
            AchievementType::CodeGolf500Point => "Earn 500 points in Code Golf",
            AchievementType::CodeGolf1000Point => "Earn 1000 points in Code Golf",
            AchievementType::CodeGolf2000Point => "Earn 2000 points in Code Golf",
            AchievementType::RestrictedSource1Point => "Earn your first point in restricted source",
            AchievementType::RestrictedSource250Point => "Earn 250 points in restricted source",
            AchievementType::RestrictedSource500Point => "Earn 500 points in restricted source",
            AchievementType::RestrictedSource1000Point => "Earn 1000 points in restricted source",
            AchievementType::RestrictedSource2000Point => "Earn 2000 points in restricted source",
            AchievementType::Python1000Point => "Earn 1000 points in python",
            AchievementType::JavaScript1000Point => "Earn 1000 points in Node.js",
            AchievementType::Rust1000Point => "Earn 1000 points in Rust",
            AchievementType::Vyxal1000Point => "Earn 1000 points in Vyxal 3",
            AchievementType::C1000Point => "Earn 1000 points in C",
            AchievementType::Apl1000Point => "Earn 1000 points in APL",
            AchievementType::StarTheRepo => "Star the Byte Heist Github Repo",
        }
    }

    pub fn get_icon(self) -> String {
        let mut hash = DefaultHasher::new();
        self.hash(&mut hash);
        let data = hash.finish() ^ 0xe249a61525b124c2;
        let data_scrambled = data ^ data << 32;
        let data_scrambled = data_scrambled ^ data_scrambled >> 6;
        let mut data_scrambled = data_scrambled ^ data_scrambled >> 16;

        macro_rules! scramble_data {
            ($value:literal) => {{
                let e = data_scrambled % $value;
                data_scrambled /= $value;
                data_scrambled ^= data << 48;
                e
            }};
        }

        let color = scramble_data!(5) as usize;
        let colors = ["red", "green", "#2222ff", "#aaaa00", "teal"];
        let cx = scramble_data!(16) * 4;
        let cy = scramble_data!(16) * 4;
        let r = scramble_data!(4) * 6 + 4;

        let p0_x = scramble_data!(8) * 4;
        let p0_y = scramble_data!(8) * 4;
        let p1_x = scramble_data!(8) * 4 + 32;
        let p1_y = scramble_data!(8) * 4;
        let p2_x = scramble_data!(8) * 4 + 16;
        let p2_y = p0_y + scramble_data!(8) * 4 + 16;

        let rect_x = scramble_data!(18) * 2 + 4;
        let rect_y = scramble_data!(18) * 2 + 4;
        let width = scramble_data!(3) * 12 + 8;
        let height = width + scramble_data!(3) - 1;

        let _ = data_scrambled;

        format!(
            r##"
                <svg viewBox="0 0 64 64" width="32" height="32">
                    <circle cx="{cx}" cy="{cy}" r="{r}" fill="{}"/>
                    <path d="M {p0_x} {p0_y} L {p1_x} {p1_y} L {p2_x} {p2_y} Z" fill="{}"/>
                    <rect x="{rect_x}" y="{rect_y}" width="{width}" height="{height}" fill="{}"/>
                    <rect x="0" y="y" width="64" height="64" fill="none" stroke-width="4" stroke="#ffffff40"/>
                </svg>
            "##,
            colors[color],
            colors[(color + 1) % colors.len()],
            colors[(color + 2) % colors.len()]
        )
    }
}

pub async fn award_achievements(pool: &PgPool) -> Result<(), sqlx::Error> {
    award_point_based_cheevos(pool).await?;
    award_misc_achievements(pool)
        .await
        .inspect_err(|e| eprintln!("Failed to award github achievement: {e:?}"))
        .unwrap();
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
