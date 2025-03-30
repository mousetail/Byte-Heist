use std::convert::Infallible;

use axum::{extract::FromRequestParts, http::request::Parts};

use super::html_context::HtmlContext;

pub enum Format {
    Json,
    Html(Box<HtmlContext>),
}

impl<S: Send + Sync> FromRequestParts<S> for Format {
    #[doc = " If the extractor fails it\'ll use this \"rejection\" type. A rejection is"]
    #[doc = " a kind of error that can be converted into a response."]
    type Rejection = Infallible;

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
            return Ok(Format::Html(Box::new(
                HtmlContext::from_request_parts(parts, state).await?,
            )));
        }
    }
}
