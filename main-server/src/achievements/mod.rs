mod points_based;

use std::hash::{DefaultHasher, Hash, Hasher};

use points_based::award_point_based_cheevos;
use sqlx::{PgPool, query_scalar};
use strum::{EnumString, IntoStaticStr, VariantArray};

#[derive(Copy, Clone, PartialEq, Eq, Hash, VariantArray, IntoStaticStr, EnumString)]
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
    // StarTheRepo,
    // SubmitAPullRequest,
    // Miscellaneous
    // SolveImpossible,
}

impl AchievementType {
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

        let color = scramble_data!(3) as usize;
        let colors = ["red", "green", "blue"];
        let cx = scramble_data!(16) * 4;
        let cy = scramble_data!(16) * 4;
        let r = scramble_data!(4) * 6 + 4;

        let p0_x = scramble_data!(8) * 4;
        let p0_y = scramble_data!(8) * 4;
        let p1_x = scramble_data!(8) * 4 + 32;
        let p1_y = scramble_data!(8) * 4;
        let p2_x = scramble_data!(8) * 4 + 16;
        let p2_y = scramble_data!(8) * 4 + 32;

        let rect_x = scramble_data!(18) * 2 + 4;
        let rect_y = scramble_data!(18) * 2 + 4;
        let width = scramble_data!(3) * 9 + 4;
        let height = width + scramble_data!(3) - 1;

        let _ = data_scrambled;

        format!(
            r#"
                <svg viewBox="0 0 64 64" width="32" height="32">
                    <circle cx="{cx}" cy="{cy}" r="{r}" fill="{}"/>
                    <path d="M {p0_x} {p0_y} L {p1_x} {p1_y} L {p2_x} {p2_y} Z" fill="{}"/>
                    <rect x="{rect_x}" y="{rect_y}" width="{width}" height="{height}" fill="{}"/>
                </svg>
            "#,
            colors[color],
            colors[(color + 1) % colors.len()],
            colors[(color + 2) % colors.len()]
        )
    }
}

pub async fn award_achievements(pool: &PgPool) -> Result<(), sqlx::Error> {
    award_point_based_cheevos(pool).await
}

pub async fn get_unread_achievements_for_user(
    pool: &PgPool,
    user_id: i32,
) -> Result<Vec<String>, sqlx::Error> {
    Ok(query_scalar!(
        "UPDATE achievements
        SET read=true
        WHERE user_id=$1 AND read=false
        RETURNING achievement",
        user_id
    )
    .fetch_all(pool)
    .await?)
}
