mod all_solutions;
mod leaderboard;
mod new_solution;
pub mod post_mortem;

pub use all_solutions::all_solutions;
pub use leaderboard::get_leaderboard;
pub use new_solution::new_solution;

use axum::{extract::Path, response::Redirect, Extension};
use serde::{Deserialize, Serialize};
use sqlx::{query_scalar, PgPool};

use crate::{
    error::Error,
    models::{account::Account, solutions::RankingMode},
    slug::Slug,
};

#[derive(Serialize, Deserialize)]
pub struct SolutionQueryParameters {
    #[serde(default)]
    ranking: RankingMode,
}

pub async fn challenge_redirect(
    Path(id): Path<i32>,
    account: Option<Account>,
    pool: Extension<PgPool>,
) -> Result<Redirect, Error> {
    challenge_redirect_no_slug(Path((id, None)), account, pool).await
}

pub async fn challenge_redirect_with_slug(
    Path((id, _slug)): Path<(i32, String)>,
    account: Option<Account>,
    pool: Extension<PgPool>,
) -> Result<Redirect, Error> {
    challenge_redirect_no_slug(Path((id, None)), account, pool).await
}

pub async fn challenge_redirect_no_slug(
    Path((id, language)): Path<(i32, Option<String>)>,
    account: Option<Account>,
    Extension(pool): Extension<PgPool>,
) -> Result<Redirect, Error> {
    let language = match language.as_ref() {
        Some(language) => language.as_str(),
        None => match account.as_ref() {
            Some(account) => account.preferred_language.as_str(),
            None => "python",
        },
    };

    let Some(slug) = query_scalar!("SELECT name FROM challenges WHERE id=$1", id)
        .fetch_optional(&pool)
        .await
        .map_err(Error::Database)?
    else {
        return Err(Error::NotFound);
    };

    Ok(Redirect::temporary(&format!(
        "/challenge/{id}/{}/solve/{language}",
        Slug(&slug)
    )))
}
