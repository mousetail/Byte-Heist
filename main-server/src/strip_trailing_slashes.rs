use std::borrow::Cow;

use axum::http::request::Parts;

use crate::error::Error;

pub async fn strip_trailing_slashes(parts: Parts) -> Result<(), Error> {
    let path = parts.uri.path();
    if path.ends_with('/') {
        return Err(Error::Redirect(Cow::Owned(
            path.trim_end_matches('/').to_string(),
        )));
    }

    Err(Error::NotFound)
}
