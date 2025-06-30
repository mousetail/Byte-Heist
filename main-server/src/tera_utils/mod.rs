use axum::{
    body::Body,
    response::{IntoResponse, Response},
};
use html_context::HtmlContext;
use macros::HtmlRenderer;
use reqwest::StatusCode;
use serde::Serialize;
use std::error::Error;
use tera::{escape_html, Context};

pub mod auto_input;
mod get_tera;
mod html_context;
mod vite;

fn render_html_error(title: &str, error: &tera::Error) -> Response {
    let message = match &error.kind {
        tera::ErrorKind::Msg(e) => format!("{e}\n{:?}", error.source()),
        _ => format!("{:#?}", error.kind),
    };
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("Content-Type", "text/html")
        .body(Body::from(format!(
            "<h2>{}</h2>\n<pre>{}</pre>",
            escape_html(title),
            escape_html(&message)
        )))
        .unwrap()
}

#[derive(Clone)]
pub struct TeraHtmlRenderer;

impl<S: Send + Sync> HtmlRenderer<S> for TeraHtmlRenderer {
    type Context = HtmlContext;
    type Err = crate::error::Error;

    fn render(
        &self,
        data: impl Serialize,
        html_context: Self::Context,
        template: &'static str,
        status_code: axum::http::StatusCode,
    ) -> Response {
        let tera = match get_tera::get_tera() {
            Ok(tera) => tera,
            Err(e) => return e.into_response(),
        };

        let mut context = Context::new();
        context.insert("object", &data);
        context.insert("account", &html_context.account);
        context.insert("dev", &cfg!(debug_assertions));

        let html = match tera.render(template, &context) {
            Ok(html) => html,
            Err(err) => return render_html_error("Error rendering template", &err),
        };
        Response::builder()
            .status(status_code)
            .header("Content-Type", "text/html")
            .body(axum::body::Body::from(html))
            .unwrap()
    }

    fn render_error(&self, err: Self::Err) -> Response {
        err.into_response()
    }

    fn render_json(&self, data: impl Serialize, status_code: axum::http::StatusCode) -> Response {
        let Ok(data) = serde_json::to_vec(&data) else {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "text/plain")
                .body(Body::from(
                    "An error was encountered serializing the response to this route".to_string(),
                ))
                .unwrap();
        };

        Response::builder()
            .status(status_code)
            .header("Content-Type", "application/json")
            .body(Body::from(data))
            .unwrap()
    }
}
