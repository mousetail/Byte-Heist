use axum::{body::Body, http::Response, response::IntoResponse};
use serde::{Serialize, Serializer};

fn serialize_error<S: Serializer>(
    error: &impl ToString,
    serializer: S,
) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> {
    serializer.serialize_str(&error.to_string())
}

#[derive(Debug, Serialize)]
pub enum RunProcessError {
    NonZeroStatusCode(#[allow(unused)] Option<i32>),
    SerializationFailed(#[serde(serialize_with = "serialize_error")] serde_json::Error),
    IOError(
        #[allow(unused)]
        #[serde(serialize_with = "serialize_error")]
        std::io::Error,
    ),
}

impl From<std::io::Error> for RunProcessError {
    fn from(value: std::io::Error) -> Self {
        RunProcessError::IOError(value)
    }
}

#[derive(Debug, Serialize)]
pub enum RunLangError {
    PluginInstallFailure(#[allow(unused)] RunProcessError),
    RunLang(#[allow(unused)] RunProcessError),
    IOError(
        #[allow(unused)]
        #[serde(serialize_with = "serialize_error")]
        std::io::Error,
    ),
    SemaphoreError(
        #[allow(unused)]
        #[serde(serialize_with = "serialize_error")]
        tokio::sync::AcquireError,
    ),
}

impl From<std::io::Error> for RunLangError {
    fn from(value: std::io::Error) -> Self {
        RunLangError::IOError(value)
    }
}

impl IntoResponse for RunLangError {
    fn into_response(self) -> axum::response::Response {
        Response::builder()
            .status(503)
            .body(Body::from(format!("{self:?}")))
            .unwrap()
    }
}
