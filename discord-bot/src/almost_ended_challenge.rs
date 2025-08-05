
use common::urls::get_url_for_challenge;
use serenity::all::{ChannelId, CreateEmbed, CreateMessage};
use sqlx::PgPool;

use crate::get_challenge_name_by_id;

pub async fn on_almost_ended_challenge(
    http: &serenity::http::Http,
    pool: &PgPool,
    channel: ChannelId,
    challenge_id: i32,
) {
    let name = get_challenge_name_by_id(pool, challenge_id).await.unwrap();

    match channel
        .send_message(
            http,
            CreateMessage::new().add_embed(gen_embed(challenge_id, name)),
        )
        .await
    {
        Ok(_) => (),
        Err(e) => eprintln!(
            "Failed to send 24 hour advance notice message for challenge {challenge_id}: {e:}"
        ),
    };
}

fn gen_embed(challenge_id: i32, challenge_name: String) -> CreateEmbed {
    let public_url = std::env::var("BYTE_HEIST_PUBLIC_URL").unwrap();
    CreateEmbed::new()
        .title(format!("{challenge_name} is ending in less than 24 hours"))
        .color(3426654)
        .description("Last chance to earn points")
        .url(format!(
            "{public_url}{}",
            get_url_for_challenge(
                challenge_id,
                Some(&challenge_name),
                common::urls::ChallengePage::Solve { language: None }
            )
        ))
}
