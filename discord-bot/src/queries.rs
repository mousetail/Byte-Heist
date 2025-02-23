use serenity::all::{ChannelId, MessageId};
use sqlx::PgPool;

use crate::{channel_id_to_i64, message_id_to_i64, LastMessage, ScoreImproved};

pub(crate) async fn get_challenge_name_by_id(
    pool: &PgPool,
    id: i32,
) -> Result<String, sqlx::Error> {
    sqlx::query_scalar!(
        "
        SELECT name
        FROM challenges
        WHERE id=$1
        ",
        id
    )
    .fetch_one(pool)
    .await
}

pub(crate) struct BasicAccontInfo {
    pub(crate) username: String,
    pub(crate) avatar: String,
}

pub(crate) struct NewScore {
    pub(crate) username: String,
    pub(crate) user_id: i32,
    pub(crate) score: i32,
}

pub(crate) async fn get_user_info_by_id(
    pool: &PgPool,
    id: i32,
) -> Result<BasicAccontInfo, sqlx::Error> {
    sqlx::query_as!(
        BasicAccontInfo,
        "SELECT username, avatar
        FROM accounts
        WHERE id=$1
        ",
        id
    )
    .fetch_one(pool)
    .await
}

pub(crate) async fn get_last_message_for_challenge(
    pool: &PgPool,
    challenge: i32,
    language: &str,
) -> Result<Option<LastMessage>, sqlx::Error> {
    sqlx::query_as!(
        LastMessage,
        r#"
        SELECT discord_messages.id,
            discord_messages.language,
            discord_messages.author as author_id,
            discord_messages.challenge as challenge_id,
            accounts.username as author_name,
            discord_messages.previous_author as previous_author_id,
            discord_messages.score as score,
            previous_account.username as "previous_author_name?",
            discord_messages.previous_author_score,
            discord_messages.message_id,
            discord_messages.channel_id
        FROM discord_messages
        INNER JOIN accounts ON discord_messages.author = accounts.id
        LEFT JOIN accounts as previous_account ON discord_messages.previous_author = previous_account.id
        WHERE discord_messages.language=$1 AND discord_messages.challenge=$2
        "#,
        language,
        challenge,
    ).fetch_optional(pool).await
}

pub(crate) async fn get_last_posted_message_id(pool: &PgPool) -> Result<Option<i32>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT id
        FROM discord_messages
        ORDER BY message_id DESC
        LIMIT 1
    "#
    )
    .fetch_optional(pool)
    .await
}

pub(crate) async fn save_new_message_info(
    pool: &PgPool,
    last_message: Option<LastMessage>,
    message: ScoreImproved,
    message_id: MessageId,
    last_author_id: Option<i32>,
    last_score: Option<i32>,
    final_channel_id: ChannelId,
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
                message_id_to_i64(message_id),
                channel_id_to_i64(final_channel_id),
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
                message_id_to_i64(message_id),
                channel_id_to_i64(final_channel_id)
            )
            .execute(pool)
            .await
        }
    }?;
    Ok(())
}
