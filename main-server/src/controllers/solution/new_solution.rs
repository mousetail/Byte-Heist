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

async fn insert_new_solution(
    pool: &PgPool,
    language_name: &str,
    version: &str,
    challenge_id: i32,
    code: &str,
    account_id: i32,
    new_score: i32,
    runtime: f32,
) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO solutions (
            language,
            version,
            challenge, 
            code,
            author, 
            score, 
            last_improved_date,
            runtime
        ) values ($1, $2, $3, $4, $5, $6, $7, $8)",
        language_name,
        version,
        challenge_id,
        code,
        account_id,
        new_score,
        OffsetDateTime::now_utc(),
        runtime
    )
    .execute(pool)
    .await
    .map_err(Error::Database)?;

    Ok(())
}

async fn update_solution(
    pool: &PgPool,
    solution: &NewSolution,
    new_score: i32,
    previous_solution_code: &Code,
    runtime: f32,
) -> Result<(), Error> {
    sqlx::query!(
        "UPDATE solutions SET 
            code=$1,
            score=$2,
            valid=true,
            validated_at=now(),
            last_improved_date=$3,
            runtime=$4
        WHERE id=$5",
        solution.code,
        new_score,
        if new_score < previous_solution_code.score || !previous_solution_code.valid {
            OffsetDateTime::now_utc()
        } else {
            previous_solution_code.last_improved_date
        },
        runtime,
        previous_solution_code.id
    )
    .execute(pool)
    .await
    .map_err(Error::Database)?;

    Ok(())
}

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
                insert_new_solution(&pool, &language_name, version, challenge_id, &solution.code, account.id, new_score, test_result.runtime).await?;
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
                update_solution(&pool, &solution, new_score, &w, test_result.runtime).await?;

                if new_score < w.score {
                    tokio::spawn(
                        post_updated_score(pool.clone(), bot, challenge_id, account.id, language_name.clone(), new_score, challenge.challenge.challenge.status)
                    );
                }

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
