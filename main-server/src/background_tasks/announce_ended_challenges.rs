use std::time::Duration;

use sqlx::{query_as, PgPool};
use tokio::time::sleep;

use crate::discord::DiscordEventSender;

struct EndedChallenge {
    id: i32,
}

async fn announce_ended_challenges(pool: &PgPool, bot: &DiscordEventSender) {
    let ended_challenges = match query_as!(
        EndedChallenge,
        r#"
            UPDATE challenges SET post_mortem_announced=true WHERE post_mortem_date > now() AND NOT post_mortem_announced RETURNING id
        "#
    ).fetch_all(pool).await {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Error feching ended challenges: {err:?}");
            return;
        }
    };

    for challenge in ended_challenges {
        eprintln!("challenge ended: {}, preparing to announce", challenge.id);
        bot.send(crate::discord::DiscordEvent::EndedChallenge {
            challenge_id: challenge.id,
        })
        .await
        .unwrap();

        sleep(Duration::from_secs(60)).await;
    }
}

async fn announce_almost_ended_challenges(pool: &PgPool, bot: &DiscordEventSender) {
    let almost_ended_challenges = match query_as!(
        EndedChallenge,
        r#"
            UPDATE challenges SET post_mortem_warning_announced=true WHERE post_mortem_date + interval '25 hours'  > now() AND NOT post_mortem_warning_announced RETURNING id
        "#
    ).fetch_all(pool).await {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Error feching ended challenges: {err:?}");
            return;
        }
    };

    for challenge in almost_ended_challenges {
        eprintln!(
            "challenge almost ended: {}, preparing to announce",
            challenge.id
        );
        bot.send(crate::discord::DiscordEvent::AlmostEndedChallenge {
            challenge_id: challenge.id,
        })
        .await
        .unwrap();

        sleep(Duration::from_secs(60)).await;
    }
}

pub async fn announce_ended_challenges_task(pool: PgPool, bot: DiscordEventSender) {
    loop {
        announce_ended_challenges(&pool, &bot).await;
        announce_almost_ended_challenges(&pool, &bot).await;

        sleep(Duration::from_secs(60 * 60 /* one hour */)).await;
    }
}
