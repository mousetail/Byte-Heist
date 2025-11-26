use std::borrow::Cow;

use axum::{
    Extension,
    extract::{Path, Query},
};
use common::langs::LANGS;
use serde::{Serialize, Serializer};
use sqlx::PgPool;
use time::OffsetDateTime;

use crate::{
    error::Error,
    models::{
        GetById,
        account::Account,
        challenge::ChallengeWithAuthorInfo,
        solutions::{Code, LeaderboardEntry, RankingMode, ScoreInfo},
    },
    test_case_formatting::OutputDisplay,
};

use super::SolutionQueryParameters;

#[derive(Serialize)]
pub struct ImprovedScoreToast {
    pub old_scores: Option<ScoreInfo>,
    pub new_scores: ScoreInfo,
}

fn serialize_date_as_timestamp<S: Serializer>(
    value: &Option<OffsetDateTime>,
    s: S,
) -> Result<S::Ok, S::Error> {
    value.map(|i| i.unix_timestamp()).serialize(s)
}

#[derive(Serialize)]
pub struct AllSolutionsOutput {
    pub(super) challenge: ChallengeWithAuthorInfo,
    pub(super) leaderboard: Vec<LeaderboardEntry>,
    pub(super) tests: Option<OutputDisplay>,
    pub(super) code: Cow<'static, str>,
    pub(super) previous_solution_invalid: bool,
    pub(super) language: String,
    pub(super) ranking: RankingMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) toast: Option<ImprovedScoreToast>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) account_id: Option<i32>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_date_as_timestamp"
    )]
    pub(super) last_improved_date: Option<OffsetDateTime>,
}

pub async fn all_solutions(
    Path((challenge_id, _slug, language_name)): Path<(i32, String, String)>,
    Query(SolutionQueryParameters { ranking }): Query<SolutionQueryParameters>,
    account: Option<Account>,
    Extension(pool): Extension<PgPool>,
) -> Result<AllSolutionsOutput, Error> {
    let leaderboard = LeaderboardEntry::get_leaderboard_near(
        &pool,
        challenge_id,
        &language_name,
        account.as_ref().map(|e| e.id),
        ranking,
    )
    .await
    .map_err(Error::Database)?;

    let challenge = ChallengeWithAuthorInfo::get_by_id(&pool, challenge_id)
        .await
        .map_err(Error::Database)?
        .ok_or(Error::NotFound)?;
    let code = match account {
        Some(ref account) => {
            Code::get_best_code_for_user(&pool, account.id, challenge_id, &language_name).await
        }
        None => None,
    };

    Ok(AllSolutionsOutput {
        challenge,
        leaderboard,
        tests: None,
        previous_solution_invalid: code.as_ref().is_some_and(|e| !e.valid),
        last_improved_date: code.as_ref().map(|i| i.last_improved_date),
        code: match code {
            Some(e) => Cow::Owned(e.code),
            None => Cow::Borrowed(
                LANGS
                    .get(&language_name)
                    .ok_or(Error::NotFound)?
                    .placeholder_text,
            ),
        },
        language: language_name,
        ranking,
        toast: None,
        account_id: account.map(|i| i.id),
    })
}
