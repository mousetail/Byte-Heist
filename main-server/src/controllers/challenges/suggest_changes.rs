use common::RunLangOutput;
use serde::Deserialize;
use sqlx::{PgPool, query, query_as, query_scalar};

use crate::{
    error::Error, models::challenge, test_case_display::OutputDisplay, test_solution::test_solution,
};

struct ChallengeFieldsNeededForValidation {
    judge: String,
    example_code: String,
}

impl ChallengeFieldsNeededForValidation {
    async fn get_by_id(pool: &PgPool, id: i32) -> Result<Self, sqlx::Error> {
        query_as!(
            ChallengeFieldsNeededForValidation,
            "
            SELECT judge, example_code
            FROM challenges
            WHERE id=$1
            ",
            id
        )
        .fetch_one(pool)
        .await
    }
}

async fn get_previous_description(pool: &PgPool, id: i32) -> Result<String, sqlx::Error> {
    query_scalar!(
        "
        SELECT description
        FROM challenges
        WHERE id=$1
        ",
        id
    )
    .fetch_one(pool)
    .await
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(sqlx::Type)]
#[sqlx(type_name = "challenge_diff_field", rename_all = "kebab-case")]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum DiffField {
    Judge,
    Example,
    Description,
}

#[derive(Deserialize)]
pub(super) struct CommentDiff {
    field: DiffField,
    replacement_value: String,
}

async fn has_pending_diff(
    pool: &PgPool,
    challenge_id: i32,
    field: DiffField,
) -> Result<Option<i32>, sqlx::Error> {
    query_scalar!(
        "
        SELECT id FROM challenge_change_suggestions
        WHERE challenge=$1
        AND field=$2::challenge_diff_field
        AND status='active'
        ",
        challenge_id,
        field as DiffField
    )
    .fetch_optional(pool)
    .await
}

async fn insert_diff(
    pool: &PgPool,
    comment_id: i32,
    challenge_id: i32,
    previous_value: String,
    diff: &CommentDiff,
) -> Result<(), sqlx::Error> {
    query!(
        r#"
            INSERT INTO challenge_change_suggestions (
                challenge,
                comment,
                old_value,
                new_value,
                field,
                status
            ) VALUES
            (
                $1,
                $2,
                $3,
                $4,
                $5::challenge_diff_field,
                'active'
            )
        "#,
        challenge_id,
        comment_id,
        previous_value,
        diff.replacement_value,
        diff.field as DiffField
    )
    .execute(pool)
    .await
    .map(|_| ())
}

pub(super) struct InsertDiffTask<'a> {
    diff: &'a CommentDiff,
    challenge_id: i32,
    previous_value: String,
}

impl<'a> InsertDiffTask<'a> {
    pub(super) async fn apply(self, pool: &PgPool, comment_id: i32) -> Result<(), Error> {
        insert_diff(
            pool,
            comment_id,
            self.challenge_id,
            self.previous_value,
            self.diff,
        )
        .await
        .map_err(Error::Database)
    }
}

pub(super) async fn handle_diff<'a>(
    pool: &PgPool,
    challenge_id: i32,
    diff: &'a CommentDiff,
) -> Result<Result<InsertDiffTask<'a>, OutputDisplay>, Error> {
    if has_pending_diff(pool, challenge_id, diff.field)
        .await
        .map_err(Error::Database)?
        .is_some()
    {
        return Err(Error::Conflict);
    }

    let (test_results, previous_value) = match diff.field {
        DiffField::Judge | DiffField::Example => {
            let mut challenge = ChallengeFieldsNeededForValidation::get_by_id(pool, challenge_id)
                .await
                .map_err(Error::Database)?;

            let previous_value = match diff.field {
                DiffField::Judge => {
                    std::mem::replace(&mut challenge.judge, diff.replacement_value.clone())
                }
                DiffField::Example => {
                    std::mem::replace(&mut challenge.example_code, diff.replacement_value.clone())
                }
                _ => unreachable!(),
            };

            let test_results = test_solution(
                &challenge.example_code,
                "nodejs",
                "22.4.0",
                &challenge.judge,
            )
            .await?;

            (Some(test_results), previous_value)
        }
        DiffField::Description => (
            None,
            get_previous_description(pool, challenge_id)
                .await
                .map_err(Error::Database)?,
        ),
    };

    if let Some(result) = test_results
        && (result.timed_out || !result.tests.pass)
    {
        return Ok(Err(result.into()));
    };

    Ok(Ok(InsertDiffTask {
        diff,
        previous_value,
        challenge_id,
    }))
}
