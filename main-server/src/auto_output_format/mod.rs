mod auto_input;
mod format;
mod get_tera;
mod html_context;
mod rejection;
mod vite;

pub use auto_input::AutoInput;
pub use format::Format;
use html_context::HtmlContext;
use std::error::Error;

use axum::{body::Body, http::Response, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde::Serialize;
use tera::{escape_html, Context};

pub struct AutoOutputFormat<T: Serialize> {
    data: T,
    format: Format,
    template: &'static str,
    status: StatusCode,
}

fn render_html_error(title: &str, error: &tera::Error) -> Response<Body> {
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

impl<T: Serialize> AutoOutputFormat<T> {
    pub fn new(data: T, template: &'static str, format: Format) -> Self {
        AutoOutputFormat {
            data,
            format,
            template,
            status: StatusCode::OK,
        }
    }

    pub fn with_status(self, status: StatusCode) -> Self {
        AutoOutputFormat { status, ..self }
    }

    fn create_html_response(
        data: T,
        template: &'static str,
        status: StatusCode,
        html_context: &HtmlContext,
    ) -> axum::response::Response {
        let tera = match get_tera::get_tera() {
            Ok(tera) => tera,
            Err(e) => return e,
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
            .status(status)
            .header("Content-Type", "text/html")
            .body(axum::body::Body::from(html))
            .unwrap()
    }

    fn create_json_response(&self) -> axum::response::Response {
        let mut response = Json(&self.data).into_response();
        *response.status_mut() = self.status;
        response
    }
}

impl<T: Serialize> IntoResponse for AutoOutputFormat<T> {
    fn into_response(self) -> axum::response::Response {
        match self.format {
            Format::Html(context) => {
                Self::create_html_response(self.data, self.template, self.status, &context)
            }
            Format::Json => self.create_json_response(),
        }
    }
}
