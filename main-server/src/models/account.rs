use std::time::Duration;

use axum::{
    body::Body,
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::{request::Parts, Response},
    response::IntoResponse,
    Extension,
};
use reqwest::StatusCode;
use serde::Serialize;
use sqlx::{prelude::FromRow, types::time::OffsetDateTime, PgPool};
use tower_sessions::Session;

use crate::{controllers::auth::ACCOUNT_ID_KEY, error::Error};

#[derive(FromRow, Serialize)]
pub struct Account {
    pub id: i32,
    pub username: String,
    pub avatar: String,
    pub preferred_language: String,
    pub admin: bool,
    pub has_solved_a_challenge: bool,
    pub last_creation_action: OffsetDateTime
}

impl Account {
    pub async fn get_by_id(pool: &PgPool, id: i32) -> Option<Self> {
        sqlx::query_as!(
            Account,
            r#"SELECT
                id, username, avatar, preferred_language, admin, last_creation_action,
                EXISTS(SELECT * FROM solutions WHERE author=$1) as "has_solved_a_challenge!"
            FROM accounts
            WHERE id=$1"#,
            id
        )
        .fetch_optional(pool)
        .await
        .unwrap()
    }

    pub async fn save_preferred_language(
        &self,
        pool: &PgPool,
        preferred_language: &str,
    ) -> Result<(), Error> {
        sqlx::query!(
            "UPDATE accounts SET preferred_language=$1 WHERE id=$2",
            preferred_language,
            self.id
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        Ok(())
    }

    pub async fn rate_limit(&self, pool: &PgPool) -> Result<(), Error> {
        if (OffsetDateTime::now_utc() - self.last_creation_action) < Duration::from_secs(60) {
            return Err(Error::RateLimit)
        }
        sqlx::query!(
            "UPDATE accounts SET last_creation_action=NOW() WHERE id=$1",
            self.id
        ).execute(pool).await.map_err(Error::Database)?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum AccountFetchError {
    SessionLoadFailed,
    NoSession,
    NotLoggedIn,
    NoAccountFound,
    DatabaseLoadFailed,
}

impl IntoResponse for AccountFetchError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AccountFetchError::NotLoggedIn => Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from(
                    r#"<h2>Not authorized</h2>
                    <p>You must be logged in to perform this action</p>

                    <a href="/login/github">Login</a>
                "#,
                ))
                .unwrap(),
            e => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "text/plain")
                .body(Body::from(format!("{e:#?}")))
                .unwrap(),
        }
    }
}

impl<S: Send + Sync> OptionalFromRequestParts<S> for Account {
    type Rejection = AccountFetchError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        match <Account as FromRequestParts<S>>::from_request_parts(parts, state).await {
            Ok(e) => Ok(Some(e)),
            Err(AccountFetchError::NoSession | AccountFetchError::NotLoggedIn) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Account {
    type Rejection = AccountFetchError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| AccountFetchError::SessionLoadFailed)?;
        let Extension(pool) =
            <Extension<PgPool> as FromRequestParts<S>>::from_request_parts(parts, state)
                .await
                .map_err(|_| AccountFetchError::DatabaseLoadFailed)?;

        match session
            .get(ACCOUNT_ID_KEY)
            .await
            .map_err(|_| AccountFetchError::NoSession)?
        {
            Some(account_id) => {
                if let Some(account) = Account::get_by_id(&pool, account_id).await {
                    return Ok(account);
                }
                Err(AccountFetchError::NoAccountFound)
            }
            _ => Err(AccountFetchError::NotLoggedIn),
        }
    }
}
