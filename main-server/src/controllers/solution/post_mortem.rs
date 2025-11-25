use axum::{Extension, extract::Path, response::Redirect};
use serde::Serialize;
use sqlx::{PgPool, query_as};

use crate::{
    error::Error,
    models::{GetById, account::Account, challenge::ChallengeWithAuthorInfo},
};

#[derive(Serialize)]
pub struct PostMortemViewOutput {
    solutions: Vec<PostMortemSolutionView>,
    id: i32,
    name: String,
    author: i32,
    language: String,
    description: String,
}

#[derive(Serialize)]
struct PostMortemSolutionView {
    code: Option<String>,
    author_id: i32,
    author_name: String,
    author_avatar: String,
    points: i32,
    runtime: f32,
    is_post_mortem: bool,
    valid: bool,
}

pub async fn post_mortem_view_without_language(
    Path((challenge_id, slug)): Path<(i32, String)>,
    account: Account,
) -> Redirect {
    Redirect::temporary(&format!(
        "/challenge/{challenge_id}/{slug}/solutions/{}",
        account.preferred_language
    ))
}

pub async fn post_mortem_view(
    Path((challenge_id, _slug, language_name)): Path<(i32, String, String)>,
    _account: Account,
    Extension(pool): Extension<PgPool>,
) -> Result<PostMortemViewOutput, Error> {
    let challenge = ChallengeWithAuthorInfo::get_by_id(&pool, challenge_id)
        .await
        .map_err(Error::Database)?
        .ok_or(Error::NotFound)?;

    let is_post_mortem = challenge.challenge.is_post_mortem;

    let solutions = if is_post_mortem {
        post_mortem_query(&pool, &language_name, challenge_id).await
    } else {
        pre_mortem_query(&pool, &language_name, challenge_id).await
    }
    .map_err(Error::Database)?;

    Ok(PostMortemViewOutput {
        solutions,
        id: challenge_id,
        name: challenge.challenge.challenge.name,
        description: challenge.challenge.challenge.description,
        author: challenge.challenge.author,
        language: language_name,
    })
}

async fn post_mortem_query(
    pool: &PgPool,
    language: &str,
    challenge_id: i32,
) -> Result<Vec<PostMortemSolutionView>, sqlx::Error> {
    query_as!(
        PostMortemSolutionView,
        r#"
            SELECT solutions.code,
                solutions.points,
                solutions.runtime,
                solutions.valid,
                solutions.is_post_mortem as "is_post_mortem!",
                accounts.id as author_id,
                accounts.username as author_name,
                accounts.avatar as author_avatar
            FROM solutions
                INNER JOIN accounts ON solutions.author = accounts.id
            WHERE solutions.challenge=$1 AND solutions.language=$2
            ORDER BY valid DESC, points ASC
        "#,
        challenge_id,
        language
    )
    .fetch_all(pool)
    .await
}

async fn pre_mortem_query(
    pool: &PgPool,
    language: &str,
    challenge_id: i32,
) -> Result<Vec<PostMortemSolutionView>, sqlx::Error> {
    query_as!(
        PostMortemSolutionView,
        r#"
            SELECT null as code,
                solutions.points,
                solutions.runtime,
                accounts.id as author_id,
                solutions.valid,
                solutions.is_post_mortem as "is_post_mortem!",
                accounts.username as author_name,
                accounts.avatar as author_avatar
            FROM solutions
                INNER JOIN accounts ON solutions.author = accounts.id
            WHERE solutions.challenge=$1 AND solutions.language=$2
            ORDER BY valid DESC, points ASC
        "#,
        challenge_id,
        language
    )
    .fetch_all(pool)
    .await
}
