use std::env::VarError;

use discord_bot::{Bot, ScoreImproved};
use reqwest::StatusCode;
use serde::Serialize;
use sqlx::PgPool;

use crate::{
    models::{
        account::Account,
        challenge::{ChallengeStatus, NewOrExistingChallenge},
        solutions::LeaderboardEntry,
    },
    slug::Slug,
};

#[derive(Serialize)]
pub struct WebHookRequest<'a> {
    pub content: Option<&'a str>,
    pub username: Option<&'a str>,
    pub avatar_url: Option<&'a str>,
    pub tts: Option<bool>,
    pub embeds: Option<Vec<Embed<'a>>>,
}

#[derive(Serialize)]
pub struct Embed<'a> {
    pub title: Option<&'a str>,
    pub description: Option<&'a str>,
    pub url: Option<&'a str>,
    pub color: Option<i32>,
}

#[derive(Debug)]
pub enum DiscordError {
    EnvVarNotValidUnicode,
    ClientBuild,
    Request,
    BadStatusCode(#[allow(unused)] StatusCode),
}

pub async fn post_discord_webhook(request: WebHookRequest<'_>) -> Result<(), DiscordError> {
    let webhook_url = match std::env::var("DISCORD_WEBHOOK_URL") {
        Ok(value) => value,
        Err(VarError::NotPresent) => return Ok(()),
        Err(VarError::NotUnicode(_)) => return Err(DiscordError::EnvVarNotValidUnicode),
    };

    let client = reqwest::ClientBuilder::new()
        .build()
        .map_err(|_| DiscordError::ClientBuild)?;
    let response = client
        .post(webhook_url)
        .json(&request)
        .send()
        .await
        .map_err(|_| DiscordError::Request)?;

    if !response.status().is_success() {
        let status = response.status();
        eprintln!("{}", response.text().await.unwrap());
        return Err(DiscordError::BadStatusCode(status));
    }

    Ok(())
}

pub async fn post_new_challenge(account: Account, challenge: NewOrExistingChallenge, row: i32) {
    let challenge = challenge.get_new_challenge();

    match post_discord_webhook(WebHookRequest {
        content: None,
        username: Some(&account.username),
        avatar_url: Some(&account.avatar),
        tts: None,
        embeds: Some(vec![Embed {
            title: Some(&format!("New Challenge: {}", challenge.name)),
            description: Some(&challenge.description[..100.min(challenge.description.len())]),
            url: Some(&format!(
                "https://byte-heist.com/challenge/{row}/{}/solve",
                Slug(&challenge.name)
            )),
            color: Some(255),
        }]),
    })
    .await
    {
        Ok(()) => (),
        Err(e) => {
            eprintln!("{e:?}");
        }
    };
}

pub async fn post_new_golfer(account: Account) {
    match post_discord_webhook(WebHookRequest {
        content: None,
        username: Some(&account.username),
        avatar_url: Some(&account.avatar),
        tts: None,
        embeds: Some(vec![Embed {
            title: Some(&format!("New Golfer: {}", account.username)),
            description: None,
            url: Some(&format!("https://byte-heist.com/user/{}", account.id)),
            color: Some(0xff00),
        }]),
    })
    .await
    {
        Ok(()) => (),
        Err(e) => {
            eprintln!("{e:?}");
        }
    };
}

#[allow(clippy::too_many_arguments)]
pub async fn post_updated_score(
    pool: PgPool,
    bot: Bot,
    challenge_id: i32,
    author: i32,
    language: String,
    score: i32,
    status: ChallengeStatus,
    is_post_mortem: bool,
) {
    match status {
        ChallengeStatus::Beta | ChallengeStatus::Draft | ChallengeStatus::Private => return,
        _ => (),
    }

    let top_solution = match LeaderboardEntry::get_top_entry(&pool, challenge_id, &language).await {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Failed to get top solution: {e:?}");
            return;
        }
    };
    if top_solution.is_none_or(|k| k.score == score && k.author_id == author) {
        bot.send(ScoreImproved {
            challenge_id,
            author,
            language,
            score,
            is_post_mortem,
        })
        .await;
    }
}
