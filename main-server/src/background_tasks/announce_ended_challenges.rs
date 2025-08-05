use std::time::Duration;

use sqlx::{query_as, PgPool};
use time::OffsetDateTime;
use tokio::time::sleep;

use crate::discord::DiscordEventSender;

struct EndedChallenge {
    id: i32,
    post_mortem_date: Option<OffsetDateTime>,
}

pub async fn announce_ended_challenges_task(pool: PgPool, bot: DiscordEventSender) {
    loop {
        let ended_challenges = match query_as!(
            EndedChallenge,
            r#"
                UPDATE challenges SET post_mortem_announced=true WHERE post_mortem_date > now() AND NOT post_mortem_announced RETURNING id, post_mortem_date
            "#
        ).fetch_all(&pool).await {
            Ok(e) => e,
            Err(err) => {
                eprintln!("Error feching ended challenges: {err:?}");
                continue;
            }
        };

        for challenge in ended_challenges {
            println!("challenge ended: {}, preparing to announce", challenge.id);
            bot.send(crate::discord::DiscordEvent::EndedChallenge {
                challenge_id: challenge.id,
            })
            .await
            .unwrap();
        }

        sleep(Duration::from_secs(60 * 60 /* one hour */)).await;
    }
}
