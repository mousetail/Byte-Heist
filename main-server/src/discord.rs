use std::time::Duration;

use common::{AchievementType, sql_enums::ChallengeStatus, urls::get_url_for_challenge};
use discord_bot::{
    Bot, ScoreImproved,
    change_suggestions::post_change_suggestion,
    new_challenge::{BestScore, ChallengePostAllSolutionsEvent, PostAllNewScoresReason},
    webhooks::{DiscordWebhookChannel, Embed, WebHookRequest, post_discord_webhook},
};
use sqlx::PgPool;

use crate::{
    achievements::award_achievement,
    models::{
        GetById,
        account::Account,
        challenge::ChallengeWithAuthorInfo,
        solutions::{LeaderboardEntry, SolutionWithLanguage},
    },
};

#[allow(clippy::enum_variant_names)]
pub enum DiscordEvent {
    NewGolfer {
        user_id: i32,
        referrer: Option<String>,
    },
    NewChallenge {
        challenge_id: i32,
    },
    PointsImproved {
        challenge_id: i32,
        solution_id: i32,
    },
    EndedChallenge {
        challenge_id: i32,
    },
    AlmostEndedChallenge {
        challenge_id: i32,
    },
    ChangeSuggestionSubmitted {
        comment_id: i32,
    },
    ChangeSuggestionVotedOn {
        #[allow(unused)]
        comment_id: i32,
    },
}

#[derive(Clone)]
pub struct DiscordEventSender(tokio::sync::mpsc::Sender<DiscordEvent>);

impl DiscordEventSender {
    pub fn new(pool: PgPool, bot: Option<Bot>) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(24);
        tokio::spawn(listen_for_events(receiver, pool, bot));
        DiscordEventSender(sender)
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
            DiscordEvent::NewGolfer { user_id, referrer } => {
                post_new_golfer(&pool, user_id, referrer).await
            }
            DiscordEvent::NewChallenge { challenge_id } => {
                post_new_challenge(&pool, challenge_id).await;

                if let Some(bot) = &bot {
                    post_best_scores_for_challenge(
                        &pool,
                        bot,
                        challenge_id,
                        PostAllNewScoresReason::NewChallenge,
                    )
                    .await;
                }
            }
            DiscordEvent::PointsImproved {
                challenge_id,
                solution_id,
            } => {
                if let Some(bot) = &bot {
                    post_updated_score(&pool, challenge_id, solution_id, bot).await
                }
            }
            DiscordEvent::EndedChallenge { challenge_id } => {
                if let Some(bot) = &bot {
                    post_best_scores_for_challenge(
                        &pool,
                        bot,
                        challenge_id,
                        PostAllNewScoresReason::EndedChallenge,
                    )
                    .await;
                }
            }
            DiscordEvent::AlmostEndedChallenge { challenge_id } => {
                if let Some(bot) = &bot {
                    bot.on_almost_ended_challenge(challenge_id).await
                }
            }
            DiscordEvent::ChangeSuggestionSubmitted { comment_id } => {
                if let Err(e) = post_change_suggestion(&pool, comment_id).await {
                    eprintln!("Database error: {e:?}");
                }
            }
            DiscordEvent::ChangeSuggestionVotedOn { comment_id: _ } => {
                // if let Err(e) = edit_change_suggestion(&pool, comment_id).await {
                //     eprintln!("Database error: {e:?}");
                // }
            }
        }
    }
}

async fn post_best_scores_for_challenge(
    pool: &PgPool,
    bot: &Bot,
    challenge_id: i32,
    reason: PostAllNewScoresReason,
) {
    let leaderboard = SolutionWithLanguage::get_best_per_language(pool, challenge_id)
        .await
        .unwrap();

    let challenge = ChallengeWithAuthorInfo::get_by_id(pool, challenge_id)
        .await
        .unwrap()
        .unwrap();

    bot.post_all_scores_for_challenge(ChallengePostAllSolutionsEvent {
        challenge_id,
        challenge_name: challenge.challenge.challenge.name,
        scores: leaderboard
            .into_iter()
            .map(|k| BestScore {
                author_id: k.author,
                author_name: k.author_name,
                language: k.language,
                score: k.points,
            })
            .collect(),
        reason,
    })
    .await;
}

async fn post_new_challenge(pool: &PgPool, challenge_id: i32) {
    /*
    We sleep a moment for a race condition where this runs before the transaction to add the challenge completes
    */
    tokio::time::sleep(Duration::from_secs(1)).await;

    let challenge = ChallengeWithAuthorInfo::get_by_id(pool, challenge_id)
        .await
        .unwrap()
        .unwrap();

    match post_discord_webhook(
        DiscordWebhookChannel::NewChallenge,
        WebHookRequest {
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
                    "{}",
                    get_url_for_challenge(
                        challenge_id,
                        Some(&challenge.challenge.challenge.name),
                        common::urls::ChallengePage::Solve { language: None }
                    ),
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
}

async fn post_new_golfer(pool: &PgPool, user_id: i32, referrer: Option<String>) {
    /*
    We sleep a moment for a race condition where this runs before the transaction to add the user completes
    */
    tokio::time::sleep(Duration::from_secs(1)).await;

    let Some(account) = Account::get_by_id(pool, user_id).await else {
        eprintln!(
            "Wanted to post a discord message regarding new golfer {user_id} but no such account was found"
        );
        return;
    };

    let description = referrer.map(|i| format!("Referrer: {i}"));
    match post_discord_webhook(
        DiscordWebhookChannel::NewGolfer,
        WebHookRequest {
            content: None,
            username: Some(&account.username),
            avatar_url: Some(&account.avatar),
            tts: None,
            embeds: Some(vec![Embed {
                title: Some(&format!("New Golfer: {}", account.username)),
                description: description.as_deref(),
                url: Some(&format!("https://byte-heist.com/user/{}", account.id)),
                color: Some(0xff00),
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
}

async fn award_achievements(
    pool: &PgPool,
    challenge_id: i32,
    top_solution: &Option<LeaderboardEntry>,
    solution: &SolutionWithLanguage,
) -> Result<(), sqlx::Error> {
    if solution.is_post_mortem {
        award_achievement(
            pool,
            solution.author,
            AchievementType::SolvePostMortem,
            Some(challenge_id),
            Some(&solution.language),
        )
        .await?;

        if top_solution
            .as_ref()
            .is_none_or(|e| e.author_id == solution.author && e.is_post_mortem)
        {
            award_achievement(
                pool,
                solution.author,
                AchievementType::SolvePostMortem,
                Some(challenge_id),
                Some(&solution.language),
            )
            .await?;
        }
    } else {
        if top_solution.is_none() {
            award_achievement(
                pool,
                solution.author,
                AchievementType::OnlySolution,
                Some(challenge_id),
                Some(&solution.language),
            )
            .await?;
        }

        if top_solution
            .as_ref()
            .is_none_or(|e| e.points <= solution.points)
        {
            award_achievement(
                pool,
                solution.author,
                AchievementType::FirstPlace,
                Some(challenge_id),
                Some(&solution.language),
            )
            .await?;
        }

        if top_solution
            .as_ref()
            .is_none_or(|e| e.author_id == solution.author)
        {
            award_achievement(
                pool,
                solution.author,
                AchievementType::UncontestedFirstPlace,
                Some(challenge_id),
                Some(&solution.language),
            )
            .await?;
        }
    }

    Ok(())
}

async fn post_updated_score(pool: &PgPool, challenge_id: i32, solution_id: i32, bot: &Bot) {
    let challenge = match ChallengeWithAuthorInfo::get_by_id(pool, challenge_id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            eprintln!(
                "Attempted to post updated score for challenge {challenge_id}, but challenge with id {solution_id} could not be found in the database"
            );
            return;
        }
        Err(e) => {
            eprintln!(
                "Attempted to post updated score, but got an error trying to fetch the challenge from the database: {e:?}"
            );
            return;
        }
    };
    let solution = match SolutionWithLanguage::get_by_id(pool, solution_id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            eprintln!(
                "Attempted to post updated score for challenge {challenge_id}, but solution with id {solution_id} could not be found in the database"
            );
            return;
        }
        Err(e) => {
            eprintln!(
                "Attempted to post updated score, but got an error trying to fetch the solution from the database: {e:?}"
            );
            return;
        }
    };

    match challenge.challenge.challenge.status {
        ChallengeStatus::Beta => {
            if let Err(e) = award_achievement(
                pool,
                solution.author,
                AchievementType::SolveBeta,
                Some(challenge_id),
                None,
            )
            .await
            {
                eprintln!("Can not award achievement: {e:?}");
            }
            return;
        }
        ChallengeStatus::Draft | ChallengeStatus::Private => return,
        _ => (),
    }

    let top_solution =
        match LeaderboardEntry::get_top_entry(pool, challenge_id, &solution.language).await {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Failed to get top solution: {e:?}");
                return;
            }
        };

    if let Err(e) = award_achievements(pool, challenge_id, &top_solution, &solution).await {
        eprintln!("Failed to award achievements: {e:?}");
    }

    if top_solution.is_none_or(|k| k.points == solution.points && k.author_id == solution.author) {
        bot.on_score_improved(ScoreImproved {
            challenge_id,
            author: solution.author,
            language: solution.language,
            score: solution.points,
            is_post_mortem: solution.is_post_mortem,
        })
        .await;
    }
}
