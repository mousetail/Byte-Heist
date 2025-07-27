mod view_challenge;

use axum::{extract::Path, http::StatusCode, Extension};
use common::urls::get_url_for_challenge;
use macros::CustomResponseMetadata;
use serde::Serialize;
use sqlx::PgPool;

pub use view_challenge::{post_comment, post_reaction, view_challenge};

use crate::{
    discord::DiscordEventSender,
    error::Error,
    models::{
        account::Account,
        challenge::{
            ChallengeCategory, ChallengeStatus, ChallengeWithAuthorInfo, ChallengeWithTests,
            HomePageChallenge, NewChallenge, NewOrExistingChallenge,
        },
        solutions::InvalidatedSolution,
        GetById,
    },
    solution_invalidation::notify_challenge_updated,
    tera_utils::auto_input::AutoInput,
    test_solution::test_solution,
};

async fn do_invalid_solutions_exist(
    pool: &PgPool,
    account: &Option<Account>,
) -> Result<bool, Error> {
    Ok(if let Some(account) = account {
        InvalidatedSolution::invalidated_solution_exists(account.id, pool)
            .await
            .map_err(Error::Database)?
    } else {
        false
    })
}

#[derive(Serialize)]
pub struct HomePageChallengesOutput {
    public_challenges: Vec<HomePageChallenge>,
    invalid_solutions_exist: bool,
}

pub async fn get_homepage(
    Extension(pool): Extension<PgPool>,
    account: Option<Account>,
) -> Result<HomePageChallengesOutput, Error> {
    let public_challenges =
        HomePageChallenge::get_all_by_status(&pool, ChallengeStatus::Public, &account, 8).await?;

    let invalid_solutions_exist = do_invalid_solutions_exist(&pool, &account).await?;

    Ok(HomePageChallengesOutput {
        public_challenges,
        invalid_solutions_exist,
    })
}

#[derive(Serialize)]
pub struct AllChallengesOutput {
    public_challenges: Vec<HomePageChallenge>,
    beta_challenges: Vec<HomePageChallenge>,
    invalid_solutions_exist: bool,
}

pub async fn all_challenges(
    Extension(pool): Extension<PgPool>,
    account: Option<Account>,
) -> Result<AllChallengesOutput, Error> {
    let public_challenges =
        HomePageChallenge::get_all_by_status(&pool, ChallengeStatus::Public, &account, 1000)
            .await?;
    let beta_challenges =
        HomePageChallenge::get_all_by_status(&pool, ChallengeStatus::Beta, &account, 1000).await?;

    let invalid_solutions_exist = do_invalid_solutions_exist(&pool, &account).await?;

    Ok(AllChallengesOutput {
        public_challenges,
        beta_challenges,
        invalid_solutions_exist,
    })
}

pub async fn compose_challenge(
    id: Option<Path<(i32, String)>>,
    pool: Extension<PgPool>,
) -> Result<NewOrExistingChallenge, Error> {
    let challenge = match id {
        None => NewOrExistingChallenge::default(),
        Some(Path((id, _))) => {
            let Some(o) = NewOrExistingChallenge::get_by_id(&pool, id).await? else {
                return Err(Error::NotFound);
            };
            o
        }
    };

    Ok(challenge)
}

pub async fn new_challenge(
    id: Option<Path<(i32, String)>>,
    Extension(pool): Extension<PgPool>,
    Extension(bot): Extension<DiscordEventSender>,
    account: Account,
    AutoInput(challenge): AutoInput<NewChallenge>,
) -> Result<CustomResponseMetadata<ChallengeWithTests>, Error> {
    account.rate_limit(&pool).await?;

    let (new_challenge, existing_challenge) = match id {
        Some(Path((id, _))) => {
            let existing_challenge = ChallengeWithAuthorInfo::get_by_id(&pool, id)
                .await
                .map_err(Error::Database)?
                .ok_or(Error::NotFound)?;
            let mut new_challenge = existing_challenge.clone();
            new_challenge.challenge.challenge = challenge.clone();
            (
                NewOrExistingChallenge::Existing(new_challenge),
                Some(existing_challenge),
            )
        }
        None => (NewOrExistingChallenge::New(challenge), None),
    };

    let challenge = new_challenge.get_new_challenge();

    if let Err(e) = challenge.validate(
        existing_challenge.as_ref().map(|k| &k.challenge),
        account.admin,
    ) {
        return Ok(CustomResponseMetadata::new(ChallengeWithTests {
            challenge: new_challenge,
            tests: None,
            validation: Some(e),
        })
        .with_status(StatusCode::BAD_REQUEST));
    }

    let tests = test_solution(
        &challenge.example_code,
        "nodejs",
        "22.4.0",
        &challenge.judge,
    )
    .await
    .inspect_err(|e| eprintln!("{e:?}"))
    .map_err(|_| Error::ServerError)?;

    if !tests.tests.pass {
        return Ok(CustomResponseMetadata::new(ChallengeWithTests {
            challenge: new_challenge,
            tests: Some(tests.into()),
            validation: None,
        })
        .with_status(StatusCode::BAD_REQUEST));
    }

    match id {
        None => {
            let row = sqlx::query_scalar!(
                r#"
                INSERT INTO challenges (name, judge, description, author, status, category)
                values ($1, $2, $3, $4, $5::challenge_status, $6::challenge_category)
                RETURNING id"#,
                challenge.name,
                challenge.judge,
                challenge.description,
                account.id,
                challenge.status as ChallengeStatus,
                challenge.category as ChallengeCategory,
            )
            .fetch_one(&pool)
            .await
            .map_err(Error::Database)?;

            let destination = format!(
                "{}",
                get_url_for_challenge(
                    row,
                    Some(&challenge.name),
                    common::urls::ChallengePage::Edit
                )
            );

            if challenge.status == ChallengeStatus::Public {
                bot.send(crate::discord::DiscordEvent::NewChallenge { challenge_id: row })
                    .await
                    .unwrap();
            }

            Err(Error::Redirect(std::borrow::Cow::Owned(destination)))
        }
        Some(Path((id, _slug))) => {
            let existing_challenge = existing_challenge.unwrap(); // This can never fail

            if !account.admin && existing_challenge.challenge.author != account.id {
                return Err(Error::PermissionDenied(
                    "You don't have permission to edit this challenge",
                ));
            }

            if &existing_challenge.challenge.challenge != challenge {
                sqlx::query!(
                    r"UPDATE challenges
                    SET
                        name=$1,
                        judge=$2, 
                        description=$3, 
                        example_code=$4, 
                        status=$5::challenge_status, 
                        category=$6::challenge_category,
                        post_mortem_date=COALESCE(
                            challenges.post_mortem_date,
                            CASE
                                WHEN $5::challenge_status!='public' THEN NULL
                                WHEN $6::challenge_category='restricted-source' THEN now() + INTERVAL '2 months'
                                WHEN $6::challenge_category='code-golf' THEN now() + INTERVAL '6 months'
                                ELSE NULL
                            END
                        )

                    WHERE id=$7",
                    challenge.name,
                    challenge.judge,
                    challenge.description,
                    challenge.example_code,
                    challenge.status as ChallengeStatus,
                    challenge.category as ChallengeCategory,
                    id
                )
                .execute(&pool)
                .await
                .unwrap();

                // Tells the solution invalidator task to re-check all solutions
                notify_challenge_updated();

                if existing_challenge.challenge.challenge.status != ChallengeStatus::Public
                    && challenge.status == ChallengeStatus::Public
                {
                    bot.send(crate::discord::DiscordEvent::NewChallenge { challenge_id: id })
                        .await
                        .unwrap();
                }
            }

            Ok(CustomResponseMetadata::new(ChallengeWithTests {
                challenge: new_challenge,
                tests: Some(tests.into()),
                validation: None,
            }))
        }
    }
}
