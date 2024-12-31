use axum::{
    extract::{Path, Query},
    Extension,
};
use common::langs::LANGS;
use discord_bot::Bot;
use reqwest::StatusCode;
use sqlx::{types::time::OffsetDateTime, PgPool};

use crate::{
    auto_output_format::{AutoInput, AutoOutputFormat, Format},
    discord::post_updated_score,
    error::Error,
    models::{
        account::Account,
        challenge::ChallengeWithAuthorInfo,
        solutions::{Code, LeaderboardEntry, NewSolution},
    },
    test_solution::test_solution,
};

use super::{all_solutions::AllSolutionsOutput, SolutionQueryParameters};

pub async fn new_solution(
    Path((challenge_id, _slug, language_name)): Path<(i32, String, String)>,
    Query(SolutionQueryParameters { ranking }): Query<SolutionQueryParameters>,
    account: Account,
    Extension(pool): Extension<PgPool>,
    Extension(bot): Extension<Bot>,
    format: Format,
    AutoInput(solution): AutoInput<NewSolution>,
) -> Result<AutoOutputFormat<AllSolutionsOutput>, Error> {
    let version = LANGS
        .get(&language_name)
        .ok_or(Error::NotFound)?
        .latest_version;

    account
        .save_preferred_language(&pool, &language_name)
        .await?;

    let challenge = ChallengeWithAuthorInfo::get_by_id(&pool, challenge_id)
        .await?
        .ok_or(Error::NotFound)?;

    let test_result = test_solution(
        &solution.code,
        &language_name,
        version,
        &challenge.challenge.challenge.judge,
    )
    .await?;

    let previous_code =
        Code::get_best_code_for_user(&pool, account.id, challenge_id, &language_name).await;

    let previous_solution_invalid =
        !test_result.tests.pass && previous_code.as_ref().is_some_and(|e| !e.valid);

    let status = if test_result.tests.pass {
        // Currently the web browser turns all line breaks into "\r\n" when a solution
        // is submitted. This should eventually be fixed in the frontend, but for now
        // we just replace "\r\n" with "\n" when calculating the score to make it match
        // the byte counter in the editor.
        // Related: https://github.com/mousetail/Byte-Heist/issues/34
        let new_score = (solution.code.len() - solution.code.matches("\r\n").count()) as i32;

        match previous_code {
            None => {
                sqlx::query!(
                    "INSERT INTO solutions (
                        language,
                        version,
                        challenge, 
                        code,
                        author, 
                        score, 
                        last_improved_date
                    ) values ($1, $2, $3, $4, $5, $6, $7)",
                    language_name,
                    version,
                    challenge_id,
                    solution.code,
                    account.id,
                    new_score,
                    OffsetDateTime::now_utc()
                )
                .execute(&pool)
                .await
                .map_err(Error::Database)?;

                tokio::spawn(
                    post_updated_score(pool.clone(), bot, challenge_id, account.id, language_name.clone(), new_score, challenge.challenge.challenge.status)
                );

                StatusCode::CREATED
            }
            Some(w) if
                // Always replace an invalid solution
                !w.valid
                // Replace a solution if the score is better
                || w.score >= new_score => {
                sqlx::query!(
                    "UPDATE solutions SET 
                        code=$1,
                        score=$2,
                        valid=true,
                        validated_at=now(),
                        last_improved_date=$3
                    WHERE id=$4",
                    solution.code,
                    new_score,
                    if new_score < w.score || !w.valid {
                        OffsetDateTime::now_utc()
                    } else {
                        w.last_improved_date
                    },
                    w.id
                )
                .execute(&pool)
                .await
                .map_err(Error::Database)?;

                tokio::spawn(
                    post_updated_score(pool.clone(), bot, challenge_id, account.id, language_name.clone(), new_score, challenge.challenge.challenge.status)
                );

                StatusCode::CREATED
            }
            Some(_) => {
                // This means the code passed but is not better than the previously saved solution
                // So we don't save
                StatusCode::OK
            },
        }
    } else {
        StatusCode::BAD_REQUEST
    };
    Ok(AutoOutputFormat::new(
        AllSolutionsOutput {
            challenge,
            leaderboard: LeaderboardEntry::get_leaderboard_near(
                &pool,
                challenge_id,
                &language_name,
                Some(account.id),
                ranking,
            )
            .await
            .map_err(Error::Database)?,
            tests: Some(test_result.into()),
            code: Some(solution.code),
            language: language_name,
            previous_solution_invalid,
            ranking,
        },
        "challenge.html.jinja",
        format,
    )
    .with_status(status))
}
