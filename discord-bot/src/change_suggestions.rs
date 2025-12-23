use common::{
    diff_tools::inline_diff,
    sql_enums::{ChallengeCategory, ChallengeStatus},
    urls::get_url_for_challenge,
};
use sqlx::{PgPool, query_as};

use crate::webhooks::{DiscordWebhookChannel, Embed, WebHookRequest, post_discord_webhook};

struct ChangeSuggestionInfo {
    old_value: String,
    new_value: String,
    challenge_id: i32,
    challenge_name: String,
    challenge_category: ChallengeCategory,
    challenge_status: ChallengeStatus,
    #[allow(unused)]
    author_id: i32,
    author_username: String,
    author_avatar: String,
}

impl ChangeSuggestionInfo {
    async fn get(pool: &PgPool, id: i32) -> Result<Self, sqlx::Error> {
        query_as!(
            ChangeSuggestionInfo,
            r#"
                SELECT
                    challenge_change_suggestions.old_value,
                    challenge_change_suggestions.new_value,
                    challenges.id as challenge_id,
                    challenges.name as challenge_name,
                    challenges.status as "challenge_status!:ChallengeStatus",
                    challenges.category as "challenge_category!:ChallengeCategory",
                    accounts.id as author_id,
                    accounts.username as author_username,
                    accounts.avatar as author_avatar
                FROM challenge_change_suggestions
                INNER JOIN challenge_comments AS comments ON comments.id=challenge_change_suggestions.comment
                INNER JOIN accounts ON accounts.id=comments.author
                INNER JOIN challenges ON challenges.id=comments.challenge
                WHERE comments.id=$1
            "#,
            id
        ).fetch_one(pool)
        .await
    }
}

pub async fn post_change_suggestion(pool: &PgPool, comment_id: i32) -> Result<(), sqlx::Error> {
    let ChangeSuggestionInfo {
        old_value,
        new_value,
        challenge_id,
        challenge_name,
        challenge_status,
        challenge_category,
        author_id: _,
        author_username,
        author_avatar,
    } = ChangeSuggestionInfo::get(pool, comment_id).await?;

    if matches!(
        challenge_status,
        ChallengeStatus::Draft | ChallengeStatus::Private
    ) || matches!(challenge_category, ChallengeCategory::Private)
    {
        return Ok(());
    }

    match post_discord_webhook(
        DiscordWebhookChannel::ChangeRequest,
        WebHookRequest {
            content: None,
            username: Some(&author_username),
            avatar_url: Some(&author_avatar),
            tts: None,
            embeds: Some(vec![Embed {
                title: Some(&format!("Edit Suggested: {}", challenge_name)),
                description: Some(&inline_diff(
                    &old_value.replace("`", "`\u{200B}"),
                    &new_value.replace("`", "`\u{200B}"),
                )),
                url: Some(&format!(
                    "{}#comment-{}",
                    get_url_for_challenge(
                        challenge_id,
                        Some(&challenge_name),
                        common::urls::ChallengePage::View
                    ),
                    comment_id
                )),
                color: Some(255),
            }]),
        },
    )
    .await
    {
        Ok(()) => (),
        Err(e) => {
            eprintln!("{e:?}");
        }
    };

    Ok(())
}
