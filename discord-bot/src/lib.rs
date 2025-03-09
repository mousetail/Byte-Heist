mod checks;
mod queries;

use checks::{get_last_best_score_fields, should_post_new_message};
use common::langs::LANGS;
use queries::{
    get_challenge_name_by_id, get_last_message_for_challenge, get_last_posted_message_id,
    get_user_info_by_id, save_new_message_info, BasicAccontInfo, NewScore,
};
use serenity::all::{
    ChannelId, Colour, CreateEmbed, CreateEmbedAuthor, CreateMessage, EditMessage, MessageId,
};
use sqlx::PgPool;

pub struct ScoreImproved {
    pub challenge_id: i32,
    pub author: i32,
    pub language: String,
    pub score: i32,
    pub is_post_mortem: bool,
}

struct LastMessage {
    id: i32,
    #[allow(unused)]
    language: String,
    #[allow(unused)]
    challenge_id: i32,
    author_id: i32,
    author_name: String,
    score: i32,
    previous_author_id: Option<i32>,
    previous_author_name: Option<String>,
    previous_author_score: Option<i32>,
    message_id: i64,
    channel_id: i64,
}

fn format_message(
    new_message: &ScoreImproved,
    challenge_name: &str,
    author: &BasicAccontInfo,
    last_best_score: &NewScore,
) -> CreateEmbed {
    let public_url = std::env::var("YQ_PUBLIC_URL").unwrap();

    CreateEmbed::new()
        .title(format!(
            "Improved score for {challenge_name} in {}{}",
            LANGS
                .get(&new_message.language)
                .map(|d| d.display_name)
                .unwrap_or(&new_message.language),
            if new_message.is_post_mortem {
                " (Post mortem)"
            } else {
                ""
            }
        ))
        .author(
            CreateEmbedAuthor::new(&author.username)
                .icon_url(&author.avatar)
                .url(format!("{public_url}/user/{}", &new_message.author)),
        )
        .url(format!(
            "{}/challenge/{}/{}/{}/{}",
            public_url,
            new_message.challenge_id,
            slug::slugify(challenge_name),
            if new_message.is_post_mortem {
                "solutions"
            } else {
                "solve"
            },
            new_message.language
        ))
        .field(
            &last_best_score.username,
            format!("{}", last_best_score.score),
            true,
        )
        .field(&author.username, format!("{}", new_message.score), true)
        .color(if new_message.is_post_mortem {
            Colour::from_rgb(255, 0, 0)
        } else {
            Colour::from_rgb(0, 0, 255)
        })
}

#[derive(Debug)]
enum HandleMessageError {
    Sql(#[allow(unused)] sqlx::Error),
    Disocrd(#[allow(unused)] serenity::Error),
}

impl From<sqlx::Error> for HandleMessageError {
    fn from(value: sqlx::Error) -> Self {
        HandleMessageError::Sql(value)
    }
}

impl From<serenity::Error> for HandleMessageError {
    fn from(value: serenity::Error) -> Self {
        HandleMessageError::Disocrd(value)
    }
}

async fn post_or_edit_message(
    message_id: Option<MessageId>,
    channel_id: ChannelId,
    embed: CreateEmbed,
    http_client: &serenity::http::Http,
) -> Result<serenity::all::Message, serenity::Error> {
    match message_id {
        Some(e) => {
            channel_id
                .edit_message(http_client, e, EditMessage::new().add_embed(embed))
                .await
        }
        None => {
            channel_id
                .send_message(http_client, CreateMessage::new().add_embed(embed))
                .await
        }
    }
}

async fn handle_message(
    score_improved_event: ScoreImproved,
    pool: &PgPool,
    http_client: &serenity::http::Http,
    channel_id: ChannelId,
) -> Result<(), HandleMessageError> {
    let last_message_for_challenge = get_last_message_for_challenge(
        pool,
        score_improved_event.challenge_id,
        &score_improved_event.language,
    )
    .await?;
    let challenge_name = get_challenge_name_by_id(pool, score_improved_event.challenge_id).await?;
    let user_info = get_user_info_by_id(pool, score_improved_event.author).await?;
    let latest_message = get_last_posted_message_id(pool).await?;

    let last_best_score = get_last_best_score_fields(
        &last_message_for_challenge,
        latest_message,
        NewScore {
            username: user_info.username.clone(),
            user_id: score_improved_event.author,
            score: score_improved_event.score,
        },
    );

    let formatted_message = format_message(
        &score_improved_event,
        &challenge_name,
        &user_info,
        &last_best_score,
    );
    let message_id = should_post_new_message(
        latest_message,
        &score_improved_event,
        &last_message_for_challenge,
    );
    let posted_message = post_or_edit_message(
        message_id,
        last_message_for_challenge
            .as_ref()
            .map(|k| channel_id_from_i64(k.channel_id))
            .unwrap_or(channel_id),
        formatted_message,
        http_client,
    )
    .await?;

    save_new_message_info(
        pool,
        last_message_for_challenge,
        score_improved_event,
        posted_message.id,
        Some(last_best_score.user_id),
        Some(last_best_score.score),
        channel_id,
    )
    .await?;

    Ok(())
}

pub struct Bot {
    http_client: serenity::http::Http,
    pool: PgPool,
    channel_id: ChannelId,
}

impl Bot {
    pub fn new(pool: PgPool, discord_token: String, channel_id: u64) -> Self {
        let http_client = serenity::http::Http::new(&discord_token);
        let channel_id = ChannelId::new(channel_id);

        Bot {
            http_client,
            pool,
            channel_id,
        }
    }

    pub async fn send(&self, message: ScoreImproved) {
        match handle_message(message, &self.pool, &self.http_client, self.channel_id).await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("(Partially) Failed to send discord update: {e:?}")
            }
        };
    }
}

fn message_id_from_i64(value: i64) -> MessageId {
    MessageId::new(u64::from_be_bytes(value.to_be_bytes()))
}

fn channel_id_from_i64(value: i64) -> ChannelId {
    ChannelId::new(u64::from_be_bytes(value.to_be_bytes()))
}

fn message_id_to_i64(value: MessageId) -> i64 {
    i64::from_be_bytes(value.get().to_be_bytes())
}

fn channel_id_to_i64(value: ChannelId) -> i64 {
    i64::from_be_bytes(value.get().to_be_bytes())
}
