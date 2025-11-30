use std::time::Duration;

use common::langs::LANGS;
use sqlx::{PgPool, query, query_as, query_scalar};
use tokio::time::sleep;
use tower_sessions::cookie::time::OffsetDateTime;

use crate::{achievements::award_achievement, test_solution::test_solution};

struct QueueEntry {
    id: i32,
    code: String,
    language: String,
    judge: String,
    time_out_count: i32,
}

struct SolutionRetestRequest {
    id: i32,
    language: Option<String>,
    challenge: Option<i32>,
}

static SOLUTION_INVALIATION_NOTIFICATION: tokio::sync::Notify = tokio::sync::Notify::const_new();

pub async fn solution_invalidation_task(pool: PgPool) {
    loop {
        match solution_invalidation_task_inner(&pool).await {
            Ok(()) => (),
            Err(e) => {
                eprintln!("Solution invalidation task failed: {e:?}");

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn solution_invalidation_task_inner(pool: &PgPool) -> Result<(), sqlx::Error> {
    loop {
        let Some(task) = query_as!(
            SolutionRetestRequest,
            r#"
                SELECT
                    id,
                    challenge,
                    language
                FROM solution_retest_request
                WHERE NOT processed
            "#
        )
        .fetch_optional(pool)
        .await?
        else {
            SOLUTION_INVALIATION_NOTIFICATION.notified().await;
            continue;
        };

        let solutions = query_as!(
            QueueEntry,
            r#"
                SELECT
                    solutions.id,
                    solutions.code,
                    solutions.language,
                    challenges.judge,
                    solutions.time_out_count
                FROM solutions
                INNER JOIN challenges ON solutions.challenge = challenges.id
                WHERE ($1::text IS NULL OR solutions.language=$1::text) AND
                    ($2::integer IS NULL OR challenges.id=$2::integer) AND
                    solutions.valid
            "#,
            task.language,
            task.challenge
        )
        .fetch_all(pool)
        .await?;

        eprintln!(
            "Processing task {}, number of solutions: {}",
            task.id,
            solutions.len()
        );

        let mut solutions_passed = 0;
        let mut solutions_failed = 0;
        let mut solutions_timed_out = 0;

        for solution in solutions {
            let Some(lang) = LANGS.get(&solution.language) else {
                eprintln!(
                    "Skipping solution in non-existant lang {}",
                    solution.language
                );
                continue;
            };
            let version = lang.latest_version;

            let result =
                match test_solution(&solution.code, &solution.language, version, &solution.judge)
                    .await
                {
                    Ok(e) => e,
                    Err(err) => {
                        eprintln!("{err:?}");

                        sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                };

            if result.timed_out && solution.time_out_count < 3 {
                query!(
                    "UPDATE solutions
                    SET
                        time_out_count=time_out_count+1,
                        fail_reason=$2
                    WHERE id=$1",
                    solution.id,
                    task.id
                )
                .execute(pool)
                .await?;

                solutions_timed_out += 1;
            } else if result.tests.pass {
                query!(
                    "UPDATE solutions SET validated_at=now(), valid=true, fail_reason=null, runtime=$1 WHERE id=$2",
                    result.runtime,
                    solution.id
                )
                .execute(pool)
                .await?;

                solutions_passed += 1
            } else {
                eprintln!(
                    "Solution {} in {} invalidated at {}",
                    solution.id,
                    solution.language,
                    OffsetDateTime::now_utc()
                );

                query!(
                    "UPDATE solutions SET valid=false, fail_reason=$2 WHERE id=$1",
                    solution.id,
                    task.id
                )
                .execute(pool)
                .await?;

                solutions_failed += 1;
            }

            query!(
                "INSERT INTO solution_invalidation_log(solution, pass, request, timed_out)
                VALUES ($1, $2, $3, $4)",
                solution.id,
                result.tests.pass,
                task.id,
                result.timed_out
            )
            .execute(pool)
            .await?;

            sleep(Duration::from_millis(250)).await;
        }

        query!(
            r#"
                UPDATE solution_retest_request
                SET
                    processed=true,
                    solutions_passed=$2,
                    solutions_failed=$3,
                    solutions_timed_out=$4
                WHERE id=$1
            "#,
            task.id,
            solutions_passed,
            solutions_failed,
            solutions_timed_out
        )
        .execute(pool)
        .await?;

        if solutions_failed > 0 {
            award_achievement_for_solutions_invalidation(pool, task, solutions_failed).await?;
        }

        SOLUTION_INVALIATION_NOTIFICATION.notified().await;
    }
}

async fn award_achievement_for_solutions_invalidation(
    pool: &PgPool,
    task: SolutionRetestRequest,
    number_failed: i32,
) -> Result<(), sqlx::Error> {
    let Some(author_id) = query_scalar!(
        "SELECT author FROM solution_retest_request WHERE id=$1",
        task.id
    )
    .fetch_optional(pool)
    .await?
    else {
        return Ok(());
    };

    if number_failed > 0 {
        award_achievement(
            pool,
            author_id,
            common::AchievementType::ChangeSuggestionInvalidates1,
            task.challenge,
            None,
        )
        .await?;
    }

    Ok(())
}

fn notify_challenge_updated() {
    SOLUTION_INVALIATION_NOTIFICATION.notify_one();
}

pub async fn queue_solution_retesting(
    pool: &PgPool,
    challenge_id: Option<i32>,
    language: Option<&str>,
    comment_id: Option<i32>,
    author_id: i32,
) -> Result<(), sqlx::Error> {
    query!(
        "
        INSERT INTO solution_retest_request(challenge, language, comment, author)
        VALUES ($1, $2, $3, $4)
        ",
        challenge_id,
        language,
        comment_id,
        author_id
    )
    .execute(pool)
    .await?;

    notify_challenge_updated();

    Ok(())
}
