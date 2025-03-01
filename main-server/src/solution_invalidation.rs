use std::time::Duration;

use common::langs::LANGS;
use futures_util::StreamExt;
use sqlx::{query, query_as, PgPool};
use tokio::time::sleep;
use tower_sessions::cookie::time::OffsetDateTime;

use crate::test_solution::test_solution;

struct QueueEntry {
    id: i32,
    code: String,
    language: String,
    judge: String,
}

static SOLUTION_INVALIATION_NOTIFICATION: tokio::sync::Notify = tokio::sync::Notify::const_new();

pub async fn solution_invalidation_task(pool: PgPool) {
    'outer: loop {
        let mut tasks = query_as!(
            QueueEntry,
            r#"
                SELECT solutions.id, solutions.code as code, challenges.judge as judge, solutions.language as language
                FROM solutions
                LEFT JOIN challenges ON solutions.challenge = challenges.id
                WHERE challenges.updated_at > solutions.validated_at
                AND solutions.valid = true
            "#
        )
        .fetch(&pool);

        while let Some(task) = tasks.next().await {
            let task = match task {
                Ok(task) => task,
                Err(e) => {
                    eprint!("{e:?}");
                    SOLUTION_INVALIATION_NOTIFICATION.notified().await;
                    continue 'outer;
                }
            };

            let Some(lang) = LANGS.get(&task.language) else {
                eprintln!("Skipping solution in non-existant lang {}", task.language);
                continue;
            };
            let version = lang.latest_version;

            let result = match test_solution(&task.code, &task.language, version, &task.judge).await
            {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("{err:?}");

                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            if result.tests.pass {
                query!(
                    "UPDATE solutions SET validated_at=now(), runtime=$1 WHERE id=$2",
                    result.runtime,
                    task.id
                )
                .execute(&pool)
                .await
                .unwrap();
            } else {
                println!(
                    "Solution {} in {} invalidated at {}",
                    task.id,
                    task.language,
                    OffsetDateTime::now_utc()
                );

                query!("UPDATE solutions SET valid=false WHERE id=$1", task.id)
                    .execute(&pool)
                    .await
                    .unwrap();
            }

            query!(
                "INSERT INTO solution_invalidation_log(solution, pass)
                VALUES ($1, $2)",
                task.id,
                result.tests.pass
            )
            .execute(&pool)
            .await
            .unwrap();

            sleep(Duration::from_millis(250)).await;
        }

        SOLUTION_INVALIATION_NOTIFICATION.notified().await;
    }
}

pub fn notify_challenge_updated() {
    SOLUTION_INVALIATION_NOTIFICATION.notify_one();
}
