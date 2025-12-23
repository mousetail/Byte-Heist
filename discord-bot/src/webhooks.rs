use std::env::VarError;

use reqwest::StatusCode;
use serde::Serialize;

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

pub enum DiscordWebhookChannel {
    NewGolfer,
    NewChallenge,
    ChangeRequest,
}

impl DiscordWebhookChannel {
    fn get_env_var_name(self) -> &'static str {
        match self {
            DiscordWebhookChannel::NewGolfer => "DISCORD_NEW_GOLFER_WEBHOOK_URL",
            DiscordWebhookChannel::NewChallenge => "DISCORD_NEW_CHALLENGE_WEBHOOK_URL",
            DiscordWebhookChannel::ChangeRequest => "DISCORD_CHANGE_REQUEST_WEBHOOK_URL",
        }
    }
}

pub async fn post_discord_webhook(
    channel: DiscordWebhookChannel,
    request: WebHookRequest<'_>,
) -> Result<(), DiscordError> {
    let webhook_url = match std::env::var(channel.get_env_var_name()) {
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
