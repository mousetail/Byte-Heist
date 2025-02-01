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
            tera.register_filter("format_number", format_number);
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

fn format_number(value: &Value, data: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    if !data.is_empty() {
        return Err(tera::Error::msg(
            "The format string filter takes no parameters",
        ));
    }

    match value {
        Value::Number(number) => {
            return Ok(Value::String(format_number_with_thousands_seperators(
                number.as_i64().unwrap(),
            )))
        }
        _ => return Err(tera::Error::msg(format!("Expected a number, got {value}"))),
    }
}

fn format_number_with_thousands_seperators(num: i64) -> String {
    use num_format::{Locale, ToFormattedString};

    num.to_formatted_string(&Locale::en)
}
