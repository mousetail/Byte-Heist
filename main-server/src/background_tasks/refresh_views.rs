use std::time::Duration;

use sqlx::{query, PgPool};

pub async fn refresh_views_task(pool: PgPool) {
    loop {
        let statement = query!("REFRESH MATERIALIZED VIEW scores")
            .execute(&pool)
            .await;
        match statement {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error refreshing scores: {e:?}");
            }
        }

        let statement = query!("REFRESH MATERIALIZED VIEW user_scoring_info")
            .execute(&pool)
            .await;
        match statement {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error refreshing scores: {e:?}");
            }
        }

        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}
