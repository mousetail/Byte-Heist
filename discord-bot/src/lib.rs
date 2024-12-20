mod checks;
mod queries;

use common::langs::LANGS;
use queries::{get_challenge_name_by_id, get_last_message, get_user_info_by_id, BasicAccontInfo};
use serenity::all::{
    ChannelId, CreateEmbed, CreateEmbedAuthor, CreateMessage, EditMessage, MessageId,
};
use sqlx::{pool, query_scalar, PgPool};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct ScoreImproved {
    pub challenge_id: i32,
    pub author: i32,
    pub language: String,
    pub score: i32,
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
    previous_message: &Option<LastMessage>,
    new_message: &ScoreImproved,
    challenge_name: &str,
    author: &BasicAccontInfo,
) -> CreateEmbed {
    let public_url = std::env::var("YQ_PUBLIC_URL").unwrap();

    let mut embed = CreateEmbed::new()
        .title(format!(
            "Improved score for {challenge_name} in {}",
            LANGS
                .get(&new_message.language)
                .map(|d| d.display_name)
                .unwrap_or(&new_message.language)
        ))
        .author(
            CreateEmbedAuthor::new(&author.username)
                .icon_url(&author.avatar)
                .url(format!("{public_url}/user/{}", &new_message.author)),
        )
        .url(format!(
            "{}/challenge/{}/{}/solve/{}",
            public_url,
            new_message.challenge_id,
            slug::slugify(challenge_name),
            new_message.language
        ));
    if let Some(previous) = previous_message {
        let (previous_author, previous_score) = if previous.author_id == new_message.author {
            if let (Some(previous_author_name), Some(previous_author_score)) = (
                &previous.previous_author_name,
                &previous.previous_author_score,
            ) {
                (previous_author_name.clone(), *previous_author_score)
            } else {
                (previous.author_name.clone(), previous.score)
            }
        } else {
            (previous.author_name.clone(), previous.score)
        };
        embed = embed.field(previous_author, format!("{}", previous_score), true);
    }
    embed = embed.field(&author.username, format!("{}", new_message.score), true);

    embed
}

async fn save_new_message_info(
    pool: &PgPool,
    last_message: Option<LastMessage>,
    message: ScoreImproved,
    message_id: i64,
    last_author_id: Option<i32>,
    last_score: Option<i32>,
    final_channel_id: i64,
) -> Result<(), sqlx::Error> {
    match &last_message {
        Some(e) => {
            sqlx::query!(
                "UPDATE discord_messages
                SET author=$1,
                score=$2,
                previous_author=$3,
                previous_author_score=$4,
                message_id=$5,
                channel_id=$6
                WHERE id=$7",
                message.author,
                message.score,
                last_author_id,
                last_score,
                message_id,
                final_channel_id,
                e.id
            )
            .execute(pool)
            .await
        }
        None => {
            sqlx::query!(
                r#"INSERT INTO discord_messages
                (
                    language,
                    challenge,
                    author,
                    previous_author,
                    previous_author_score,
                    score,
                    message_id,
                    channel_id
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6,
                    $7,
                    $8 
                )"#,
                message.language,
                message.challenge_id,
                message.author,
                last_author_id,
                last_score,
                message.score,
                message_id,
                final_channel_id
            )
            .execute(pool)
            .await
        }
    }?;
    Ok(())
}

#[derive(Debug)]
enum HandleMessageError {
    Sql(#[allow(unused)] sqlx::Error),
    Disocrd(#[allow(unused)] serenity::Error),
}

impl From<sqlx::Error> for HandleMessageError {
    fn from(value: sqlx::Error) -> Self {
        return HandleMessageError::Sql(value);
    }
}

impl From<serenity::Error> for HandleMessageError {
    fn from(value: serenity::Error) -> Self {
        return HandleMessageError::Disocrd(value);
    }
}

async fn handle_message(
    message: ScoreImproved,
    pool: &PgPool,
    http_client: &serenity::http::Http,
    channel_id: ChannelId,
) -> Result<(), HandleMessageError> {
    let last_message = get_last_message(&pool, message.challenge_id, &message.language).await?;
    let challenge_name = get_challenge_name_by_id(&pool, message.challenge_id).await?;
    let user_info = get_user_info_by_id(&pool, message.author).await?;

    let formatted_message = format_message(&last_message, &message, &challenge_name, &user_info);
    let (message_id, last_author_id, last_score, final_channel_id) = if let Some(k) = last_message
        .as_ref()
        .filter(|e| e.author_id == message.author)
    {
        ChannelId::new(u64::from_be_bytes(k.channel_id.to_be_bytes()))
            .edit_message(
                &http_client,
                MessageId::new(u64::from_be_bytes(k.message_id.to_be_bytes())),
                EditMessage::new().embed(formatted_message),
            )
            .await?;
        (
            k.message_id,
            k.previous_author_id,
            k.previous_author_score,
            k.channel_id,
        )
    } else {
        let response = channel_id
            .send_message(&http_client, CreateMessage::new().embed(formatted_message))
            .await?;

        let (last_author_id, last_author_score) = match &last_message {
            Some(e) => (Some(e.author_id), Some(e.score)),
            None => (None, None),
        };
        (
            i64::from_be_bytes(response.id.get().to_be_bytes()),
            last_author_id,
            last_author_score,
            i64::from_be_bytes(channel_id.get().to_be_bytes()),
        )
    };

    save_new_message_info(
        &pool,
        last_message,
        message,
        message_id,
        last_author_id,
        last_score,
        final_channel_id,
    )
    .await?;

    Ok(())
}

async fn handle_bot_queue(
    mut receiver: Receiver<ScoreImproved>,
    http_client: serenity::http::Http,
    pool: PgPool,
    channel_id: ChannelId,
) {
    match http_client.get_current_application_info().await {
        Ok(o) => {
            println!("Discord Bot Initialized, user name: {:?}", o.name)
        }
        Err(e) => {
            eprint!("Failed to initalize disocrd bot: {:?}", e);
            return;
        }
    }

    while let Some(message) = receiver.recv().await {
        match handle_message(message, &pool, &http_client, channel_id).await {
            Ok(ok) => (),
            Err(e) => {
                eprintln!("(Partially) Failed to send discord update: {e:?}")
            }
        };
    }
}

pub fn init_bot(pool: PgPool, discord_token: String, channel_id: u64) -> Sender<ScoreImproved> {
    let (sender, receiver) = tokio::sync::mpsc::channel::<ScoreImproved>(32);
    let http_client = serenity::http::Http::new(&discord_token);

    let channel = ChannelId::new(channel_id);

    tokio::spawn(handle_bot_queue(receiver, http_client, pool, channel));

    sender
}

#[derive(Clone)]
pub struct Bot {
    pub channel: Option<Sender<ScoreImproved>>,
}

impl Bot {
    pub async fn send(&self, message: ScoreImproved) {
        if let Some(channel) = &self.channel {
            if let Err(e) = channel.send(message).await {
                eprintln!("Error sending: {e:?}",);
            }
        }
    }
}
