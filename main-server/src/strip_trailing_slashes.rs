use axum::{http::request::Parts, response::Redirect};

use crate::error::Error;

pub async fn strip_trailing_slashes(parts: Parts) -> Result<Redirect, Error> {
    let path = parts.uri.path();
    if path.ends_with('/') {
        return Ok(Redirect::permanent(path.trim_end_matches('/')));
    }

    Err(Error::NotFound)
}
