use std::collections::HashMap;

use common::sql_enums::{ChallengeCategory, ChallengeDifficulty, ChallengeStatus};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, query_as, types::time::OffsetDateTime};

use crate::{error::Error, test_case_formatting::OutputDisplay};

use super::{GetById, account::Account};

#[derive(sqlx::FromRow, Deserialize, Serialize, Eq, PartialEq, Clone)]
pub struct NewChallenge {
    pub description: String,
    pub judge: String,
    pub name: String,
    pub example_code: String,
    pub category: ChallengeCategory,
    pub status: ChallengeStatus,
    pub unit: String,
}

impl NewChallenge {
    pub fn validate(
        &self,
        previous: Option<&Challenge>,
        is_admin: bool,
    ) -> Result<(), HashMap<&'static str, &'static str>> {
        let mut errors = HashMap::new();

        if previous.is_some_and(|e| e.is_post_mortem) {
            errors.insert("status", "Can't edit an ended challenge");
        }

        if self.name.is_empty() {
            errors.insert("name", "name can't be empty");
        }
        if self.description.is_empty() {
            errors.insert("description", "description can not be empty");
        }
        if self.example_code.is_empty() {
            errors.insert("example-code", "Please enter some example code that should pass your challenge. This ensures your challenge is possible and that your judge works.");
        }
        if self.judge.is_empty() || self.judge == DEFAULT_JUDGE {
            errors.insert(
                "judge",
                "Please write a judge that can validate the correctness of a solution.",
            );
        }
        if self.status == ChallengeStatus::Public
            && !is_admin
            && previous.is_none_or(|k| k.challenge.status == ChallengeStatus::Public)
        {
            errors.insert("status", "you can't make a challenge public");
        } else if self.status != ChallengeStatus::Public
            && !is_admin
            && previous.is_some_and(|k| k.challenge.status == ChallengeStatus::Public)
        {
            errors.insert(
                "status",
                "You can't make a published challenge private again",
            );
        }

        if !is_admin
            && previous.is_some_and(|k| {
                k.challenge.status == ChallengeStatus::Public
                    && k.challenge.category != self.category
            })
        {
            errors.insert("category", "can't change the category of a live challenge");
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

const DEFAULT_JUDGE: &str = concat!(
    "(async function*(context: Context): Challenge {\n",
    "\t// Single Test\n",
    "\tyield (await context.run(undefined)).assertEquals('Hello World!');\n\n",
    "\t// Automatically shuffle and deal test cases over multiple runs\n",
    "\tyield* context.runTestCases(\n",
    "\t\t[\n",
    "\t\t\t[\"Input\", \"Expected Output\"],\n",
    "\t\t]\n",
    "\t);\n",
    "\t// For \"Filter\" Style challenges where the goal is to output all inputs that match some condition\n",
    "\tyield* context.runFilterCases([\n",
    "\t\t[\"This should be outputted\", true],\n",
    "\t\t[\"This should not be outputted\", false],\n",
    "\t]);\n",
    "\t// Finally, the challenge is passed if no test cases failed\n",
    "\treturn context.noFailures();\n",
    "})"
);

impl Default for NewChallenge {
    fn default() -> Self {
        NewChallenge {
            description: concat!(
                "Explain in detail how to solve your challenge. Good challenge descriptions ",
                "include examples and links to relevent resources. Markdown is supported"
            )
            .to_string(),
            judge: DEFAULT_JUDGE.to_string(),
            name: String::new(),
            example_code: String::new(),
            category: ChallengeCategory::RestrictedSource,
            status: ChallengeStatus::Draft,
            unit: "bytes".to_string(),
        }
    }
}

#[derive(Serialize, Clone)]
#[serde(untagged)]
pub enum NewOrExistingChallenge {
    Existing(ChallengeWithAuthorInfo),
    New(NewChallenge),
}
impl NewOrExistingChallenge {
    pub fn get_new_challenge(&self) -> &NewChallenge {
        match self {
            Self::Existing(e) => &e.challenge.challenge,
            Self::New(k) => k,
        }
    }

    pub async fn get_by_id(pool: &PgPool, id: i32) -> Result<Option<Self>, Error> {
        Ok(ChallengeWithAuthorInfo::get_by_id(pool, id)
            .await
            .map_err(Error::Database)?
            .map(NewOrExistingChallenge::Existing))
    }
}

impl Default for NewOrExistingChallenge {
    fn default() -> Self {
        Self::New(NewChallenge::default())
    }
}

#[derive(Serialize)]
pub struct ChallengeWithTests {
    #[serde(flatten)]
    pub challenge: NewOrExistingChallenge,
    pub tests: Option<OutputDisplay>,
    pub validation: Option<HashMap<&'static str, &'static str>>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Clone)]
pub struct Challenge {
    pub id: Option<i32>,
    #[sqlx(flatten)]
    #[serde(flatten)]
    pub challenge: NewChallenge,
    pub author: i32,
    pub post_mortem_date: Option<OffsetDateTime>,
    pub is_post_mortem: bool,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Clone)]
pub struct HomePageChallenge {
    id: i32,
    name: String,
    category: ChallengeCategory,
    difficulty: ChallengeDifficulty,
    score: Option<i32>,
    description: String,
    post_mortem_date: Option<OffsetDateTime>,
    is_post_mortem: bool,
    submission_count: i64,
    user_has_submitted: bool,
    up_votes: Option<i64>,
    down_votes: Option<i64>,
}

impl HomePageChallenge {
    pub async fn get_all_by_status(
        pool: &PgPool,
        status: ChallengeStatus,
        user: &Option<Account>,
        limit: i64,
    ) -> Result<Vec<HomePageChallenge>, Error> {
        query_as!(
            HomePageChallenge,
            r#"
            SELECT
                challenges.id,
                name,
                category as "category!: ChallengeCategory",
                scores.score,
                difficulty as "difficulty!: ChallengeDifficulty",
                CAST(description AS varchar(120)) as "description!",
                post_mortem_date,
                COALESCE((challenges.post_mortem_date IS NOT NULL AND challenges.post_mortem_date < now()), false) as "is_post_mortem!",
                COALESCE(submission_counts.count, 0) as "submission_count!",
                COALESCE(user_submissions.has_submitted, false) as "user_has_submitted!",
                (SELECT count(*) FROM challenge_votes WHERE challenge=challenges.id and is_upvote) as up_votes,
                (SELECT count(*) FROM challenge_votes WHERE challenge=challenges.id and not is_upvote) as down_votes
            FROM challenges
            LEFT JOIN scores ON scores.author = $2 AND scores.challenge = challenges.id AND scores.language = $3
            LEFT JOIN (
                SELECT challenge, COUNT(*) as count 
                FROM solutions 
                GROUP BY challenge
            ) submission_counts ON submission_counts.challenge = challenges.id
            LEFT JOIN (
                SELECT challenge, true as has_submitted
                FROM solutions 
                WHERE author = $2
                GROUP BY challenge
            ) user_submissions ON user_submissions.challenge = challenges.id
            WHERE status=($1) AND category != 'private'
            ORDER BY challenges.go_live_date DESC,
                up_votes DESC,
                down_votes ASC,
                created_at DESC
            LIMIT $4
        "#,
            status as ChallengeStatus,
            user.as_ref().map(|i| i.id),
            user.as_ref().map(|i| &i.preferred_language),
            limit
        )
        .fetch_all(pool)
        .await
        .map_err(Error::Database)
    }
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Clone)]
pub struct ChallengeWithAuthorInfo {
    #[sqlx(flatten)]
    #[serde(flatten)]
    pub challenge: Challenge,
    pub author_name: String,
    pub author_avatar: String,
}

impl GetById for ChallengeWithAuthorInfo {
    async fn get_by_id(pool: &PgPool, id: i32) -> Result<Option<Self>, sqlx::Error> {
        let sql = "SELECT
            challenges.id,
            challenges.name,
            challenges.description,
            challenges.judge,
            challenges.example_code,
            challenges.author,
            challenges.category,
            challenges.status,
            challenges.post_mortem_date,
            challenges.unit,
            (challenges.post_mortem_date IS NOT NULL
                AND challenges.post_mortem_date < now()) as is_post_mortem,
            accounts.username as author_name,
            accounts.avatar as author_avatar
            FROM challenges LEFT JOIN accounts ON challenges.author = accounts.id
            WHERE challenges.id=$1
            "
        .to_string();

        let challenge: Option<ChallengeWithAuthorInfo> =
            sqlx::query_as(&sql).bind(id).fetch_optional(pool).await?;

        Ok(challenge)
    }
}
