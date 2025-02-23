use axum::{
    body::Body,
    extract::rejection::{FormRejection, JsonRejection},
    http::Response,
    response::IntoResponse,
};
use reqwest::StatusCode;

pub enum AutoInputRejection {
    JsonRejection(JsonRejection),
    FormRejection(FormRejection),
    BadContentType,
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
