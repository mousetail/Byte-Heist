use std::convert::Infallible;

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use serde::Serialize;

use crate::models::account::Account;

#[derive(Serialize)]
pub struct HtmlContext {
    pub(super) account: Option<Account>,
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for HtmlContext {
    type Rejection = Infallible;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let account = Account::from_request_parts(parts, state).await.ok();

        return Ok(HtmlContext { account });
    }
}
