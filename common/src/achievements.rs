use std::fmt::Write;
use std::{
    fs::read_to_string,
    hash::{DefaultHasher, Hash, Hasher},
};

use serde::Serialize;
use strum::{EnumString, IntoStaticStr, VariantArray};

use crate::langs::{LANGS, Lang};

#[derive(Serialize, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum AchievementCategory {
    PointRelated,
    LanguageRelated,
    SolveRelated,
    ChangeSuggestions,
    Miscellaneous,
}

pub enum ChallengeCategory {
    CodeGolf,
    RestrictedSource,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, VariantArray, IntoStaticStr, EnumString, Debug)]
pub enum AchievementType {
    // Solve Related
    // OnePoint,
    UncontestedFirstPlace,
    FirstPlace,
    OnlySolution,
    // FiveLanguages,
    // ImproveFirstPlace,
    FirstDaySolve,
    LastDaySolve,
    // Change Suggestion Related
    ImproveDescription,
    ImproveJudge,
    ImproveExample,
    Vote,
    ChangeSuggestionInvalidates1,
    // ChangeSuggestionInvalidate12,
    // UpvoteSuggestionThatInvalidates3,
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
    JavaScript3500Point,
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
    Contribute,
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
            AchievementType::JavaScript3500Point => "[Even more]",
            AchievementType::Rust1000Point => "[Insert Joke Here]",
            AchievementType::Vyxal1000Point => "[Insert Joke Here]",
            AchievementType::C1000Point => "Deep Blue Sea",
            AchievementType::Apl1000Point => "Original Sin",
            AchievementType::StarTheRepo => "Starstruck",
            AchievementType::FirstDaySolve => "Early Bird",
            AchievementType::LastDaySolve => "Late Bird",
            AchievementType::Contribute => "Hero",
            AchievementType::FirstPlace => "A winner is you",
            AchievementType::UncontestedFirstPlace => "The winner you are",
            AchievementType::OnlySolution => "A player be thee",
            AchievementType::ImproveDescription => "Pedant",
            AchievementType::ImproveExample => "[Insert Name Here]",
            AchievementType::ImproveJudge => "[Insert Name Here]",
            AchievementType::ChangeSuggestionInvalidates1 => "Cheese Enthusiast",
            AchievementType::Vote => "Thumbs",
        }
    }

    pub fn get_achievement_category(self) -> AchievementCategory {
        match self {
            AchievementType::StarTheRepo | AchievementType::Contribute => {
                AchievementCategory::Miscellaneous
            }
            AchievementType::Python1000Point
            | AchievementType::JavaScript1000Point
            | AchievementType::Apl1000Point
            | AchievementType::C1000Point
            | AchievementType::Rust1000Point
            | AchievementType::JavaScript3500Point
            | AchievementType::Vyxal1000Point => AchievementCategory::LanguageRelated,
            AchievementType::FirstDaySolve
            | AchievementType::LastDaySolve
            | AchievementType::FirstPlace
            | AchievementType::OnlySolution
            | AchievementType::UncontestedFirstPlace => AchievementCategory::SolveRelated,
            AchievementType::ImproveExample
            | AchievementType::ImproveJudge
            | AchievementType::ImproveDescription
            | AchievementType::ChangeSuggestionInvalidates1 => {
                AchievementCategory::ChangeSuggestions
            }
            _ => AchievementCategory::PointRelated,
        }
    }

    pub fn get_associated_number(self) -> Option<i32> {
        match self {
            Self::CodeGolf1Point | Self::RestrictedSource1Point => Some(1),
            Self::CodeGolf250Point | Self::RestrictedSource250Point => Some(250),
            Self::CodeGolf500Point | Self::RestrictedSource500Point => Some(500),
            Self::CodeGolf1000Point
            | Self::RestrictedSource1000Point
            | Self::Apl1000Point
            | Self::Python1000Point
            | Self::C1000Point
            | Self::Vyxal1000Point
            | Self::JavaScript1000Point
            | Self::Rust1000Point => Some(1000),
            Self::CodeGolf2000Point | Self::RestrictedSource2000Point => Some(2000),

            AchievementType::JavaScript3500Point => Some(3500),
            _ => None,
        }
    }

    pub fn get_achievement_description(self) -> &'static str {
        match self {
            AchievementType::OnlySolution => "Be the first to solve a challenge in a language",
            AchievementType::UncontestedFirstPlace => "Beat the best score",
            AchievementType::CodeGolf1Point => "Earn your first point in Code Golf",
            AchievementType::CodeGolf250Point => "Earn 250 points in Code Golf",
            AchievementType::CodeGolf500Point => "Earn 500 points in Code Golf",
            AchievementType::CodeGolf1000Point => "Earn 1,000 points in Code Golf",
            AchievementType::CodeGolf2000Point => "Earn 2,000 points in Code Golf",
            AchievementType::RestrictedSource1Point => "Earn your first point in restricted source",
            AchievementType::RestrictedSource250Point => "Earn 250 points in restricted source",
            AchievementType::RestrictedSource500Point => "Earn 500 points in restricted source",
            AchievementType::RestrictedSource1000Point => "Earn 1,000 points in restricted source",
            AchievementType::RestrictedSource2000Point => "Earn 2,000 points in restricted source",
            AchievementType::Python1000Point => "Earn 1,000 points in python",
            AchievementType::JavaScript1000Point => "Earn 1,000 points in Node.js",
            AchievementType::JavaScript3500Point => "Earn 3,500 points in Node.js",
            AchievementType::Rust1000Point => "Earn 1,000 points in Rust",
            AchievementType::Vyxal1000Point => "Earn 1,000 points in Vyxal 3",
            AchievementType::C1000Point => "Earn 1,000 points in C",
            AchievementType::Apl1000Point => "Earn 1,000 points in APL",
            AchievementType::StarTheRepo => "Star the Byte Heist Github Repo",
            AchievementType::FirstDaySolve => {
                "Solve a challenge within 24 hours of when it goes live"
            }
            AchievementType::LastDaySolve => "Solve a challenge less than 24 hours before it ends",
            AchievementType::Contribute => "Contribute to Byte Heist",
            AchievementType::FirstPlace => {
                "Get first place on a challenge, even if just for a moment"
            }
            AchievementType::ImproveDescription => {
                "Submit a change suggestion that updates a challenge's description and is accepted"
            }
            AchievementType::ImproveExample => {
                "Submit a change suggestion that updates a challenge's example code and is accepted"
            }
            AchievementType::ImproveJudge => {
                "Submit a change suggestion that updates a challenge's judge and is accepted"
            }
            AchievementType::ChangeSuggestionInvalidates1 => {
                "Submit a change suggestion that invalidates a solution"
            }
            AchievementType::Vote => "Vote up or down on a change suggestion",
        }
    }

    pub fn get_icon(self) -> String {
        format!("<img src=\"/static/achievement-icons/{self:?}.svg\">")
    }

    pub fn get_icon_source(self) -> String {
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

        let color = match self.get_achievement_category() {
            AchievementCategory::PointRelated => 0,
            AchievementCategory::LanguageRelated => 1,
            AchievementCategory::SolveRelated => 2,
            AchievementCategory::Miscellaneous => 3,
            AchievementCategory::ChangeSuggestions => 4,
        };

        if let Some(((icon, color), number)) = self
            .get_associated_language()
            .map(|i| (i.icon, i.color))
            .or_else(|| {
                self.get_associated_category().map(|i| match i {
                    ChallengeCategory::CodeGolf => ("../../templates/icons/clock.svg", "white"),
                    ChallengeCategory::RestrictedSource => {
                        ("../../templates/icons/heartbeat.svg", "red")
                    }
                })
            })
            .zip(self.get_associated_number())
        {
            let significant_digits = ((number << 1) - 1).ilog2().max(1);
            let side_length = (0..).find(|i| i * i >= significant_digits).unwrap();

            let mut out = format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="48" height="48" fill="{color}" color="{color}">"#
            );
            let width = 64 / side_length;

            let content = read_to_string(format!("static/lang-icons/{icon}")).unwrap();

            let symbol = format!(
                "<symbol id=\"{self:?}-root\" {}</symbol>",
                &content[4..content.len() - 6]
                    .replace(" xmlns=\"http://www.w3.org/2000/svg\"", "")
                    .replace("id=\"", &format!("id=\"{self:?}"))
                    .replace("href=\"#", &format!("href=\"#{self:?}"))
                    .replace("url(#", &format!("url(#{self:?}"))
            );
            out.push_str(&symbol);
            write!(
                out,
                r#"
                <symbol id="{self:?}-null" viewBox="0 0 32 32">
                    <circle cx="16" cy="16" r="4"/>
                </symbol>
            "#
            )
            .unwrap();

            for i in 0..significant_digits {
                let bit = number & (1 << i) > 0;

                let x = i % side_length * width;
                let y = i / side_length * width;

                write!(
                    out,
                    r##"<use href="#{self:?}-{}" x="{x}" y="{y}" width="{width}" height="{width}"/>"##,
                    if bit { "root" } else { "null" }
                )
                .unwrap();
            }
            write!(out, "<rect x=\"0\" y=\"0\" width=\"64\" height=\"64\" fill=\"none\" stroke-width=\"4\" stroke=\"#ffffff40\"/></svg>").unwrap();
            return out;
        };
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

        let rectangle_rotate = scramble_data!(7) * 2;

        let rect_x = scramble_data!(18) * 2 + 4;
        let rect_y = scramble_data!(18) * 2 + 4;
        let width = scramble_data!(3) * 12 + 8;
        let height = width + scramble_data!(3) - 1;

        let _ = data_scrambled;

        format!(
            r##"
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="32" height="32">
                    <circle cx="{cx}" cy="{cy}" r="{r}" fill="{}"/>
                    <path d="M {p0_x} {p0_y} L {p1_x} {p1_y} L {p2_x} {p2_y} Z" fill="{}"/>
                    <rect x="{rect_x}" y="{rect_y}" width="{width}" height="{height}" fill="{}" transform="rotate({rectangle_rotate})"/>
                    <rect x="0" y="0" width="64" height="64" fill="none" stroke-width="4" stroke="#ffffff40"/>
                    {}
                </svg>
            "##,
            colors[color],
            colors[(color + 1) % colors.len()],
            colors[(color + 2) % colors.len()],
            if let Some(number) = self.get_associated_number() {
                format!(r##"<text x="0" y="16" font-size="17" fill="white">{number}</text>"##)
            } else {
                String::new()
            }
        )
    }

    pub fn get_associated_language(&self) -> Option<&'static Lang> {
        match self {
            AchievementType::Apl1000Point => Some("tinyapl"),
            AchievementType::Python1000Point => Some("python"),
            AchievementType::C1000Point => Some("tcc"),
            AchievementType::Rust1000Point => Some("rust"),
            AchievementType::Vyxal1000Point => Some("vyxal3"),
            AchievementType::JavaScript1000Point | Self::JavaScript3500Point => Some("nodejs"),
            _ => None,
        }
        .and_then(|i| LANGS.get(i))
    }

    pub fn get_associated_category(&self) -> Option<ChallengeCategory> {
        match self {
            AchievementType::CodeGolf1000Point
            | AchievementType::CodeGolf1Point
            | AchievementType::CodeGolf250Point
            | AchievementType::CodeGolf2000Point
            | AchievementType::CodeGolf500Point => Some(ChallengeCategory::CodeGolf),

            AchievementType::RestrictedSource1000Point
            | AchievementType::RestrictedSource1Point
            | AchievementType::RestrictedSource250Point
            | AchievementType::RestrictedSource2000Point
            | AchievementType::RestrictedSource500Point => {
                Some(ChallengeCategory::RestrictedSource)
            }

            _ => None,
        }
    }
}
