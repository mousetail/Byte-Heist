use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::{query_as, PgPool};

use crate::{error::Error, test_case_display::OutputDisplay};

use super::account::Account;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
#[derive(sqlx::Type)]
#[sqlx(type_name = "challenge_status", rename_all = "kebab-case")]
pub enum ChallengeStatus {
    Draft,
    Private,
    Beta,
    Public,
}

impl Default for ChallengeStatus {
    fn default() -> Self {
        Self::Draft
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
#[derive(sqlx::Type)]
#[sqlx(type_name = "challenge_category", rename_all = "kebab-case")]
pub enum ChallengeCategory {
    CodeGolf,
    RestrictedSource,
    Private,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Eq, PartialEq, Clone)]
pub struct NewChallenge {
    pub description: String,
    pub judge: String,
    pub name: String,
    pub example_code: String,
    pub category: ChallengeCategory,
    pub status: ChallengeStatus,
}

impl NewChallenge {
    pub fn validate(
        &self,
        previous: Option<&NewChallenge>,
        is_admin: bool,
    ) -> Result<(), HashMap<&'static str, &'static str>> {
        let mut errors = HashMap::new();
        if self.name.is_empty() {
            errors.insert("name", "name can't be empty");
        }
        if self.description.is_empty() {
            errors.insert("description", "description can not be empty");
        }
        if self.status == ChallengeStatus::Public
            && !is_admin
            && previous.is_none_or(|k| k.status == ChallengeStatus::Public)
        {
            errors.insert("status", "you can't make a challenge public");
        } else if self.status != ChallengeStatus::Public
            && !is_admin
            && previous.is_some_and(|k| k.status == ChallengeStatus::Public)
        {
            errors.insert(
                "status",
                "You can't make a published challenge private again",
            );
        }

        if !is_admin
            && previous
                .is_some_and(|k| k.status == ChallengeStatus::Public && k.category != self.category)
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

impl Default for NewChallenge {
    fn default() -> Self {
        NewChallenge {
            description: concat!(
                "Explain in detail how to solve your challenge. Good challenge descriptions ",
                "include examples and links to relevent resources. Markdown is supported"
            )
            .to_string(),
            judge: concat!(
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
            )
            .to_string(),
            name: String::new(),
            example_code: String::new(),
            category: ChallengeCategory::RestrictedSource,
            status: ChallengeStatus::Draft,
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
            .await?
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
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Clone)]
pub struct HomePageChallenge {
    id: i32,
    name: String,
    category: ChallengeCategory,
    score: Option<i64>,
}

impl HomePageChallenge {
    pub async fn get_all_by_status(
        pool: &PgPool,
        status: ChallengeStatus,
        user: &Option<Account>,
    ) -> Result<Vec<HomePageChallenge>, Error> {
        query_as!(
            HomePageChallenge,
            r#"
            SELECT
                id,
                name,
                category as "category!: ChallengeCategory",
                scores.score
            FROM challenges
            LEFT JOIN scores ON scores.author = $2 AND scores.challenge = challenges.id AND scores.language = $3
            WHERE status=($1) AND category != 'private'
            ORDER BY challenges.created_at DESC
        "#,
            status as ChallengeStatus,
            user.as_ref().map(|i| i.id),
            user.as_ref().map(|i| &i.preferred_language)
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

impl ChallengeWithAuthorInfo {
    pub async fn get_by_id(pool: &PgPool, id: i32) -> Result<Option<Self>, Error> {
        let sql = "SELECT
            challenges.id,
            challenges.name,
            challenges.description,
            challenges.judge,
            challenges.example_code,
            challenges.author,
            challenges.category,
            challenges.status,
            accounts.username as author_name,
            accounts.avatar as author_avatar
            FROM challenges LEFT JOIN accounts ON challenges.author = accounts.id
            WHERE challenges.id=$1
            "
        .to_string();

        let challenge: Option<ChallengeWithAuthorInfo> = sqlx::query_as(&sql)
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(Error::Database)?;

        Ok(challenge)
    }
}
