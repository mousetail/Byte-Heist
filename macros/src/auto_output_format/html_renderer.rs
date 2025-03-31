use axum::{extract::FromRequestParts, http::StatusCode, response::Response};
use serde::Serialize;

pub trait HtmlRenderer<S> {
    type Context: FromRequestParts<S> + Send + Sync + 'static;
    type Err;

    fn render(
        &self,
        data: impl Serialize,
        context: Self::Context,
        template: &'static str,
        status_code: axum::http::StatusCode,
    ) -> Response;

    fn render_error(&self, err: Self::Err) -> Response;

    fn render_json(&self, data: impl Serialize, status_code: axum::http::StatusCode) -> Response;
}

pub trait IntoSerializedResponse<S, R: HtmlRenderer<S>> {
    fn into_serialized_response(
        self,
        context: R::Context,
        renderer: R,
        template: &'static str,
    ) -> Response;

    fn into_json_response(self, renderer: R) -> Response;
}

impl<S, K, R: HtmlRenderer<S>> IntoSerializedResponse<S, R> for K
where
    K: Serialize,
{
    fn into_serialized_response(
        self,
        context: R::Context,
        renderer: R,
        template: &'static str,
    ) -> Response {
        renderer.render(self, context, template, StatusCode::OK)
    }

    fn into_json_response(self, renderer: R) -> Response {
        renderer.render_json(self, StatusCode::OK)
    }
}

pub struct CustomResponseMetadata<T> {
    value: T,
    status_code: StatusCode,
}

impl<K> CustomResponseMetadata<K> {
    pub fn new(data: K) -> Self {
        CustomResponseMetadata {
            value: data,
            status_code: StatusCode::OK,
        }
    }

    pub fn with_status(self, status: StatusCode) -> Self {
        CustomResponseMetadata {
            status_code: status,
            ..self
        }
    }
}

impl<S, K, R: HtmlRenderer<S>> IntoSerializedResponse<S, R> for CustomResponseMetadata<K>
where
    K: Serialize,
{
    fn into_serialized_response(
        self,
        context: R::Context,
        renderer: R,
        template: &'static str,
    ) -> Response {
        renderer.render(self.value, context, template, self.status_code)
    }

    fn into_json_response(self, renderer: R) -> Response {
        renderer.render_json(self.value, self.status_code)
    }
}
