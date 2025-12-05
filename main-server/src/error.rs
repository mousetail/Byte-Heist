use std::borrow::Cow;

use reqwest::StatusCode;
use serde::Serialize;

#[derive(Debug)]
pub enum Error {
    NotFound,
    #[allow(clippy::enum_variant_names)]
    ServerError,
    Database(sqlx::Error),
    Oauth(OauthError),
    RunLang(Cow<'static, str>),
    PermissionDenied(&'static str),
    BadRequest(&'static str),
    Redirect(RedirectType, Cow<'static, str>),
    RateLimit,
    Conflict,
}

#[derive(Debug)]
pub enum RedirectType {
    #[allow(dead_code)]
    TemporaryPreserveMethod,
    TemporaryGet,
    Permanent,
}

#[derive(Debug)]
pub enum OauthError {
    TokenExchange,
    UserInfoFetch,
    Deserialization,
    CsrfValidation,
}

impl OauthError {
    fn get_representation(self) -> ErrorRepresentation {
        ErrorRepresentation {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            title: Cow::Borrowed("OAuth Error"),
            body: Some(Cow::Owned(format!("{self:?}"))),
            location: None,
        }
    }
}

#[derive(Serialize)]
pub struct ErrorRepresentation {
    #[serde(skip)]
    pub status_code: axum::http::StatusCode,
    pub title: Cow<'static, str>,
    pub body: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Cow<'static, str>>,
}

impl Error {
    pub fn get_representation(self) -> ErrorRepresentation {
        match self {
            Error::NotFound => ErrorRepresentation {
                status_code: StatusCode::NOT_FOUND,
                title: Cow::Borrowed("Not Found"),
                body: None,
                location: None,
            },
            Error::ServerError => ErrorRepresentation {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                title: Cow::Borrowed("Internal Server Error"),
                body: None,
                location: None,
            },
            Error::Database(e) => ErrorRepresentation {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                title: Cow::Borrowed("Database Error"),
                body: Some(Cow::Owned(format!("{e:#?}"))),
                location: None,
            },
            Error::Oauth(oauth_error) => oauth_error.get_representation(),
            Error::RunLang(s) => ErrorRepresentation {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                title: Cow::Borrowed("Lang Runner Error"),
                body: Some(s),
                location: None,
            },
            Error::Conflict => ErrorRepresentation {
                status_code: StatusCode::CONFLICT,
                title: Cow::Borrowed("Conflict"),
                body: Some(Cow::Borrowed(
                    "A race condition occurred processing this request",
                )),
                location: None,
            },
            Error::PermissionDenied(e) => ErrorRepresentation {
                status_code: StatusCode::FORBIDDEN,
                title: Cow::Borrowed("Not Authorized"),
                body: Some(Cow::Borrowed(e)),
                location: None,
            },
            Error::BadRequest(e) => ErrorRepresentation {
                status_code: StatusCode::BAD_REQUEST,
                title: Cow::Borrowed("Bad Request"),
                body: Some(Cow::Borrowed(e)),
                location: None,
            },
            Error::Redirect(redirect_type, e) => ErrorRepresentation {
                status_code: match redirect_type {
                    RedirectType::TemporaryGet => StatusCode::SEE_OTHER,
                    RedirectType::Permanent => StatusCode::PERMANENT_REDIRECT,
                    RedirectType::TemporaryPreserveMethod => StatusCode::TEMPORARY_REDIRECT,
                },
                title: Cow::Borrowed(""),
                body: None,
                location: Some(e),
            },
            Error::RateLimit => ErrorRepresentation {
                status_code: StatusCode::TOO_MANY_REQUESTS,
                title: Cow::Borrowed("Rate Limit Exceeded"),
                body: Some(Cow::Borrowed("Please wait one minute")),
                location: None,
            },
        }
    }
}
