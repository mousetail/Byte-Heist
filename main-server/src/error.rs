use std::borrow::Cow;

use axum::{
    body::Body,
    http::Response,
    response::{IntoResponse, Redirect},
};
use reqwest::StatusCode;

#[derive(Debug)]
pub enum Error {
    NotFound,
    #[allow(clippy::enum_variant_names)]
    ServerError,
    Database(sqlx::Error),
    Oauth(OauthError),
    RunLang(String),
    PermissionDenied(&'static str),
    Redirect(Cow<'static, str>),
    RateLimit,
    Conflict,
}

#[derive(Debug)]
pub enum OauthError {
    TokenExchange,
    UserInfoFetch,
    Deserialization,
    CsrfValidation,
}

impl IntoResponse for OauthError {
    fn into_response(self) -> axum::response::Response {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "Text/Plain")
            .body(Body::from(format!("{self:?}")))
            .unwrap()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::NotFound => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(
                    r#"<h2>Not Found<h2>
                    <a href="/">Back to Home</a>
                "#,
                ))
                .unwrap(),
            Error::ServerError => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap(),
            Error::Database(e) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "text/html")
                .body(Body::from(format!(
                    "Database Error: <pre>{}</pre>",
                    tera::escape_html(&format!("{e:#?}"))
                )))
                .unwrap(),
            Error::Oauth(oauth_error) => oauth_error.into_response(),
            Error::RunLang(s) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!(
                    "<h2>Lang Runner Error</h2><pre>{}</pre>",
                    tera::escape_html(&s)
                )))
                .unwrap(),
            Error::Conflict => Response::builder()
                .status(StatusCode::CONFLICT)
                .header("Content-Type", "text/html")
                .body(Body::from(
                    "<h2>Conflict</h2><p>A race condition occurred handling this request</p>",
                ))
                .unwrap(),
            Error::PermissionDenied(e) => Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::from(format!(
                    "<h2>Not Authorized</h2>
                    <p>{}</p>",
                    tera::escape_html(e)
                )))
                .unwrap(),
            Error::Redirect(e) => Redirect::to(&e).into_response(),
            Error::RateLimit => Response::builder()
                .status(StatusCode::TOO_MANY_REQUESTS)
                .header("Content-Type", "text/html")
                .body(Body::from(
                    "<h1>Rate Limit Exceeded</h1><p>Please wait one minute</p>",
                ))
                .unwrap(),
        }
    }
}
