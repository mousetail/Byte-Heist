use std::borrow::Cow;
use std::fmt::Write;

use common::urls::get_url_for_challenge;
use serenity::all::{ChannelId, CreateEmbed, CreateMessage};
use sqlx::PgPool;

use crate::{ScoreImproved, get_last_message_for_challenge, save_new_message_info};

pub enum PostAllNewScoresReason {
    NewChallenge,
    EndedChallenge,
}

pub struct ChallengePostAllSolutionsEvent {
    pub challenge_id: i32,
    pub challenge_name: String,
    pub scores: Vec<BestScore>,
    pub reason: PostAllNewScoresReason,
}

pub struct BestScore {
    pub author_id: i32,
    pub author_name: String,
    pub language: String,
    pub score: i32,
}

fn gen_embed(
    ChallengePostAllSolutionsEvent {
        challenge_name,
        challenge_id,
        scores,
        reason,
    }: &ChallengePostAllSolutionsEvent,
) -> CreateEmbed {
    let public_url = std::env::var("BYTE_HEIST_PUBLIC_URL").unwrap();
    CreateEmbed::new()
        .title(match reason {
            PostAllNewScoresReason::NewChallenge => format!("New Challenge {challenge_name}"),
            PostAllNewScoresReason::EndedChallenge => {
                format!("Challenge {challenge_name} has ended, final scores:")
            }
        })
        .color(match reason {
            PostAllNewScoresReason::NewChallenge => 512,
            PostAllNewScoresReason::EndedChallenge => 15105570,
        })
        .description(scores.iter().fold(String::new(), |mut a, i| {
            let _ = writeln!(
                a,
                "- {}: {} by [{}]({public_url}/user/{})",
                i.language, i.score, i.author_name, i.author_id
            );
            a
        }))
        .url(format!(
            "{}",
            get_url_for_challenge(
                *challenge_id,
                Some(challenge_name),
                common::urls::ChallengePage::Solve { language: None }
            )
        ))
}

pub(crate) async fn post_all_scores_for_challenge(
    http: &serenity::http::Http,
    pool: &PgPool,
    channel: ChannelId,
    event: ChallengePostAllSolutionsEvent,
) -> Result<(), Cow<'static, str>> {
    channel
        .send_message(http, CreateMessage::new().add_embed(gen_embed(&event)))
        .await
        .map_err(|e| {
            Cow::Owned(format!(
                "Failed to send message for new challenge initial scores: {e:}"
            ))
        })?;

    for score in event.scores {
        let last_message =
            get_last_message_for_challenge(pool, event.challenge_id, &score.language)
                .await
                .map_err(|e| Cow::Owned(format!("Failed to fetch last message from db: {e:?}")))?;

        save_new_message_info(
            pool,
            last_message,
            ScoreImproved {
                challenge_id: event.challenge_id,
                author: score.author_id,
                language: score.language,
                score: score.score,
                is_post_mortem: false,
            },
            None,
            None,
            None,
            channel,
        )
        .await
        .map_err(|e| Cow::Owned(format!("Failed to save updated message: {e:?}")))?;
    }

    Ok(())
}
