use std::convert::Infallible;

use axum::{extract::FromRequestParts, http::request::Parts};

pub enum Format<HtmlRendererContext> {
    Json,
    Html(HtmlRendererContext),
}

impl<C: FromRequestParts<S>, S: Send + Sync> FromRequestParts<S> for Format<C> {
    #[doc = " If the extractor fails it\'ll use this \"rejection\" type. A rejection is"]
    #[doc = " a kind of error that can be converted into a response."]
    type Rejection = C::Rejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if parts
            .uri
            .path_and_query()
            .unwrap()
            .as_str()
            .ends_with(".json")
            || parts.headers.get("accept").is_some_and(|d| {
                let bytes = d.as_bytes();
                bytes.eq_ignore_ascii_case(b"application/json")
            })
        {
            Ok(Format::Json)
        } else {
            let context = C::from_request_parts(parts, state).await?;

            return Ok(Format::Html(context));
        }
    }
}
