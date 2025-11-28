use std::time::Duration;

use sqlx::{PgPool, query};

use crate::{achievements::award_achievements, controllers::challenges::handle_reactions};

pub async fn refresh_views_task(pool: PgPool) {
    loop {
        let statement = query!("REFRESH MATERIALIZED VIEW CONCURRENTLY scores")
            .execute(&pool)
            .await;
        match statement {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error refreshing scores: {e:?}");
            }
        }

        tokio::time::sleep(Duration::from_secs(15)).await;

        let statement = query!("REFRESH MATERIALIZED VIEW CONCURRENTLY user_scoring_info")
            .execute(&pool)
            .await;
        match statement {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error refreshing scores: {e:?}");
            }
        }

        tokio::time::sleep(Duration::from_secs(15)).await;

        let statement =
            query!("REFRESH MATERIALIZED VIEW CONCURRENTLY user_scoring_info_per_language")
                .execute(&pool)
                .await;
        match statement {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error refreshing scores: {e:?}");
            }
        }

        tokio::time::sleep(Duration::from_secs(15)).await;

        let handle_reactions_result = handle_reactions(&pool).await;
        match handle_reactions_result {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error processing reactions: {e:?}");
            }
        }
        tokio::time::sleep(Duration::from_secs(30)).await;

        match award_achievements(&pool).await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error awarding achievements: {e:?}")
            }
        };
    }
}
