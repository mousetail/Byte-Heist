use axum::{async_trait, extract::FromRequest, Form, Json};
use serde::de::DeserializeOwned;

use super::rejection::AutoInputRejection;

pub struct AutoInput<T: DeserializeOwned>(pub T);

#[async_trait]
impl<T: DeserializeOwned, S: Sync + Send> FromRequest<S> for AutoInput<T> {
    type Rejection = AutoInputRejection;

    async fn from_request(
        request: axum::http::Request<axum::body::Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let content_type = request.headers().get("content-type");

        if content_type.is_some_and(|b| b.as_bytes().eq_ignore_ascii_case(b"application/json")) {
            let Json(value) = Json::<T>::from_request(request, state).await?;
            return Ok(AutoInput(value));
        } else if content_type.is_some_and(|b| {
            b.as_bytes()
                .eq_ignore_ascii_case(b"application/x-www-form-urlencoded")
        }) {
            let Form(value) = Form::<T>::from_request(request, state).await?;
            return Ok(AutoInput(value));
        } else {
            return Err(AutoInputRejection::BadContentType);
        }
    }
}
