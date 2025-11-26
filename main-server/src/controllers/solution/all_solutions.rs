use axum::{
    Extension,
    extract::{Path, Query},
};
use serde::Serialize;
use sqlx::PgPool;

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

#[derive(Serialize)]
pub struct AllSolutionsOutput {
    pub(super) challenge: ChallengeWithAuthorInfo,
    pub(super) leaderboard: Vec<LeaderboardEntry>,
    pub(super) tests: Option<OutputDisplay>,
    pub(super) code: Option<String>,
    pub(super) previous_solution_invalid: bool,
    pub(super) language: String,
    pub(super) ranking: RankingMode,
    pub(super) toast: Option<ImprovedScoreToast>,
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
        Some(account) => {
            Code::get_best_code_for_user(&pool, account.id, challenge_id, &language_name).await
        }
        None => None,
    };

    Ok(AllSolutionsOutput {
        challenge,
        leaderboard,
        tests: None,
        previous_solution_invalid: code.as_ref().is_some_and(|e| !e.valid),
        code: code.map(|d| d.code),
        language: language_name,
        ranking,
        toast: None,
    })
}
