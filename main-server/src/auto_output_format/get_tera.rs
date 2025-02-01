use std::{collections::HashMap, sync::OnceLock};

use axum::response::Response;
use common::langs::LANGS;
use tera::{to_value, Tera, Value};

use crate::markdown::MarkdownFilter;

use super::{render_html_error, vite::load_assets};

static TERA: OnceLock<tera::Result<Tera>> = OnceLock::new();

pub fn get_tera() -> Result<&'static Tera, Response> {
    let value = TERA.get_or_init(|| {
        Tera::new("templates/**/*.jinja").map(|mut tera| {
            tera.autoescape_on(vec![".html.jinja", ".xml.jinja", ".html", ".xml"]);
            tera.register_function("languages", get_langs);
            tera.register_function("modules", load_assets);
            tera.register_filter("markdown", MarkdownFilter);
            tera
        })
    });

    let tera = match value.as_ref() {
        Ok(tera) => tera,
        Err(e) => {
            return Err(render_html_error("Error initializing Tera", e));
        }
    };

    Ok(tera)
}

fn get_langs(values: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    if !values.is_empty() {
        return Err(tera::Error::msg("Get langs function takes no arguments"));
    }
    to_value(LANGS).map_err(tera::Error::json)
}
