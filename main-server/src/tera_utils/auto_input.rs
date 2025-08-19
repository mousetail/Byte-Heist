use axum::{
    body::Body,
    extract::{
        rejection::{FormRejection, JsonRejection},
        FromRequest,
    },
    response::{IntoResponse, Response},
    Form, Json,
};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;

pub enum AutoInputRejection {
    JsonRejection(JsonRejection),
    FormRejection(FormRejection),
    BadContentType,
}

impl From<JsonRejection> for AutoInputRejection {
    fn from(value: JsonRejection) -> Self {
        AutoInputRejection::JsonRejection(value)
    }
}

impl From<FormRejection> for AutoInputRejection {
    fn from(value: FormRejection) -> Self {
        AutoInputRejection::FormRejection(value)
    }
}

impl IntoResponse for AutoInputRejection {
    fn into_response(self) -> axum::response::Response {
        match self {
            AutoInputRejection::JsonRejection(json_rejection) => json_rejection.into_response(),
            AutoInputRejection::FormRejection(form_rejection) => form_rejection.into_response(),
            AutoInputRejection::BadContentType => Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "text/plain")
                .body(Body::from("Excpected a content type"))
                .unwrap(),
        }
    }
}

pub struct AutoInput<T: DeserializeOwned>(pub T);

impl<T: DeserializeOwned, S: Sync + Send> FromRequest<S> for AutoInput<T> {
    type Rejection = AutoInputRejection;

    async fn from_request(
        request: axum::http::Request<axum::body::Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let content_type = request.headers().get("content-type");

        if content_type.is_some_and(|b| b.as_bytes().eq_ignore_ascii_case(b"application/json")) {
            let Json(value) = Json::<T>::from_request(request, state).await?;
            Ok(AutoInput(value))
        } else if content_type.is_some_and(|b| {
            b.as_bytes()
                .eq_ignore_ascii_case(b"application/x-www-form-urlencoded")
        }) {
            let Form(value) = Form::<T>::from_request(request, state).await?;
            Ok(AutoInput(value))
        } else {
            Err(AutoInputRejection::BadContentType)
        }
    }
}
