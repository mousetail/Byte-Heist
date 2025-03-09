use std::env::VarError;

use discord_bot::{Bot, ScoreImproved};
use reqwest::StatusCode;
use serde::Serialize;
use sqlx::PgPool;

use crate::{
    models::{
        account::Account,
        challenge::{ChallengeStatus, ChallengeWithAuthorInfo},
        solutions::{LeaderboardEntry, SolutionWithLanguage},
        GetById,
    },
    slug::Slug,
};

pub enum DiscordEvent {
    NewGolfer { user_id: i32 },
    NewChallenge { challenge_id: i32 },
    NewBestScore { challenge_id: i32, solution_id: i32 },
}

#[derive(Clone)]
pub struct DiscordEventSender(tokio::sync::mpsc::Sender<DiscordEvent>);

impl DiscordEventSender {
    pub fn new(pool: PgPool, bot: Option<Bot>) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(24);
        tokio::spawn(listen_for_events(receiver, pool, bot));
        return DiscordEventSender(sender);
    }

    pub async fn send(
        &self,
        message: DiscordEvent,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<DiscordEvent>> {
        self.0.send(message).await
    }
}

async fn listen_for_events(
    mut receiver: tokio::sync::mpsc::Receiver<DiscordEvent>,
    pool: PgPool,
    bot: Option<Bot>,
) {
    while let Some(ev) = receiver.recv().await {
        match ev {
            DiscordEvent::NewGolfer { user_id } => post_new_golfer(&pool, user_id).await,
            DiscordEvent::NewChallenge { challenge_id } => {
                post_new_challenge(&pool, challenge_id).await
            }
            DiscordEvent::NewBestScore {
                challenge_id,
                solution_id,
            } => match &bot {
                Some(bot) => post_updated_score(&pool, challenge_id, solution_id, &bot).await,
                None => (),
            },
        }
    }
}

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

pub async fn post_new_challenge(pool: &PgPool, challenge_id: i32) {
    let challenge = ChallengeWithAuthorInfo::get_by_id(pool, challenge_id)
        .await
        .unwrap()
        .unwrap();

    match post_discord_webhook(WebHookRequest {
        content: None,
        username: Some(&challenge.author_name),
        avatar_url: Some(&challenge.author_avatar),
        tts: None,
        embeds: Some(vec![Embed {
            title: Some(&format!(
                "New Challenge: {}",
                challenge.challenge.challenge.name
            )),
            description: Some(
                &challenge.challenge.challenge.description
                    [..100.min(challenge.challenge.challenge.description.len())],
            ),
            url: Some(&format!(
                "https://byte-heist.com/challenge/{challenge_id}/{}/solve",
                Slug(&challenge.challenge.challenge.name)
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

pub async fn post_new_golfer(pool: &PgPool, user_id: i32) {
    let Some(account) = Account::get_by_id(pool, user_id).await else {
        eprintln!("Wanted to post a discord message regarding new golfer {user_id} but no such account was found");
        return;
    };
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

pub async fn post_updated_score(pool: &PgPool, challenge_id: i32, solution_id: i32, bot: &Bot) {
    let challenge = match ChallengeWithAuthorInfo::get_by_id(pool, challenge_id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            eprintln!("Attempted to post updated score for challenge {challenge_id}, but challenge with id {solution_id} could not be found in the database");
            return;
        }
        Err(e) => {
            eprintln!("Attempted to post updated score, but got an error trying to fetch the challenge from the database: {e:?}");
            return;
        }
    };
    let solution = match SolutionWithLanguage::get_by_id(pool, solution_id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            eprintln!("Attempted to post updated score for challenge {challenge_id}, but solution with id {solution_id} could not be found in the database");
            return;
        }
        Err(e) => {
            eprintln!("Attempted to post updated score, but got an error trying to fetch the solution from the database: {e:?}");
            return;
        }
    };

    match challenge.challenge.challenge.status {
        ChallengeStatus::Beta | ChallengeStatus::Draft | ChallengeStatus::Private => return,
        _ => (),
    }

    let top_solution =
        match LeaderboardEntry::get_top_entry(&pool, challenge_id, &solution.language).await {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Failed to get top solution: {e:?}");
                return;
            }
        };
    if top_solution.is_none_or(|k| k.score == solution.score && k.author_id == solution.author) {
        bot.send(ScoreImproved {
            challenge_id,
            author: solution.author,
            language: solution.language,
            score: solution.score,
            is_post_mortem: solution.is_post_mortem,
        })
        .await;
    }
}
