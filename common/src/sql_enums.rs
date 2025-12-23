use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "challenge_status", rename_all = "kebab-case")
)]
#[derive(Default)]
pub enum ChallengeStatus {
    #[default]
    Draft,
    Private,
    Beta,
    Public,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "challenge_category", rename_all = "kebab-case")
)]
pub enum ChallengeCategory {
    CodeGolf,
    RestrictedSource,
    Private,
    CodeChallenge,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "challenge_difficulty", rename_all = "kebab-case")
)]
pub enum ChallengeDifficulty {
    Easy,
    Medium,
    Hard,
}
