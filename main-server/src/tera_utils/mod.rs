use axum::{
    body::Body,
    response::{IntoResponse, Redirect, Response},
};
use html_context::{HtmlContext, RenderContext};
use macros::HtmlRenderer;
use reqwest::StatusCode;
use serde::Serialize;
use std::error::Error;
use tera::{Context, escape_html};


pub mod auto_input;
mod get_tera;
mod html_context;
mod markdown;
mod syntax_highlighting;
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
            "<h2>{}</h2>\n<pre>{:#}</pre>",
            escape_html(title),
            escape_html(&message)
        )))
        .unwrap()
}

#[derive(Clone)]
pub struct TeraHtmlRenderer;

impl TeraHtmlRenderer {
    fn render_html(
        data: impl Serialize,
        context: HtmlContext,
        status_code: axum::http::StatusCode,
        template: &'static str,
    ) -> Response {
        let tera = match get_tera::get_tera() {
            Ok(tera) => tera,
            Err(e) => return e.into_response(),
        };

        let mut tera_context = Context::new();
        tera_context.insert("object", &data);
        tera_context.insert("account", &context.account);
        tera_context.insert("dev", &cfg!(debug_assertions));

        let html = match tera.render(template, &tera_context) {
            Ok(html) => html,
            Err(err) => return render_html_error("Error rendering template", &err),
        };
        Response::builder()
            .status(status_code)
            .header("Content-Type", "text/html")
            .body(axum::body::Body::from(html))
            .unwrap()
    }

    fn render_json(data: impl Serialize, status_code: axum::http::StatusCode) -> Response {
        Response::builder()
            .status(status_code)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(serde_json::to_vec(&data).unwrap()))
            .unwrap()
    }
}

impl<S: Send + Sync> HtmlRenderer<S> for TeraHtmlRenderer {
    type Context = RenderContext;
    type Err = crate::error::Error;

    fn render(
        &self,
        data: impl Serialize,
        context: Self::Context,
        template: &'static str,
        status_code: axum::http::StatusCode,
    ) -> Response {
        match context {
            RenderContext::Html(e) => Self::render_html(data, e, status_code, template),
            RenderContext::Json => Self::render_json(data, status_code),
        }
    }

    fn render_error(&self, err: Self::Err, context: Self::Context) -> Response {
        let representation = err.get_representaiton();
        let status_code = representation.status_code;
        match context {
            html_context::Format::Json => {
                return Self::render_json(
                    representation,
                    if status_code.is_redirection() {
                        axum::http::StatusCode::IM_A_TEAPOT
                    } else {
                        status_code
                    },
                );
            }
            html_context::Format::Html(ctx) => {
                if let Some(location) = representation.location {
                    return Redirect::temporary(&location).into_response();
                }

                Self::render_html(representation, ctx, status_code, "error.html.jinja")
            }
        }
    }
}
