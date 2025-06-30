use std::{collections::HashMap, sync::OnceLock};

use axum::response::{IntoResponse, Response};
use common::langs::LANGS;
use sqlx::types::time::OffsetDateTime;
use tera::{to_value, Tera, Value};
use tower_sessions::cookie::time::format_description;

use crate::markdown::MarkdownFilter;

use super::{render_html_error, vite::load_assets};

static TERA: OnceLock<tera::Result<Tera>> = OnceLock::new();

pub enum GetTerraError {
    Initalizing(&'static tera::Error),
}

impl IntoResponse for GetTerraError {
    fn into_response(self) -> Response {
        match self {
            GetTerraError::Initalizing(error) => {
                render_html_error("Error initializing Tera", error)
            }
        }
    }
}

pub fn get_tera() -> Result<&'static Tera, GetTerraError> {
    let value = TERA.get_or_init(|| {
        Tera::new("templates/**/*.jinja").map(|mut tera| {
            tera.autoescape_on(vec![".html.jinja", ".xml.jinja", ".html", ".xml"]);
            tera.register_function("languages", get_langs);
            tera.register_function("modules", load_assets);
            tera.register_filter("markdown", MarkdownFilter);
            tera.register_filter("prepend_linebreak", prepend_line_break);
            tera.register_filter("format_number", format_number);
            tera.register_filter("format_date", format_date);
            tera.register_tester("empty", empty);
            tera
        })
    });

    let tera = match value {
        Ok(tera) => tera,
        Err(e) => {
            return Err(GetTerraError::Initalizing(e));
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
        Value::Number(number) => Ok(Value::String(format_number_with_thousands_seperators(
            number.as_i64().unwrap(),
        ))),
        _ => Err(tera::Error::msg(format!("Expected a number, got {value}"))),
    }
}

fn format_number_with_thousands_seperators(num: i64) -> String {
    use num_format::{Locale, ToFormattedString};

    num.to_formatted_string(&Locale::en)
}

fn format_date(value: &Value, data: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    if !data.is_empty() {
        return Err(tera::Error::msg(
            "The format string filter takes no parameters",
        ));
    }
    let date: OffsetDateTime = serde_json::from_value(value.clone()).map_err(tera::Error::json)?;

    let offset = (date - OffsetDateTime::now_utc()).abs();

    Ok(Value::String(if offset.whole_weeks() > 12 {
        date.format(
            &format_description::parse("[year]-[month]-[day]")
                .map_err(|_e| tera::Error::call_filter("format_date", "unkown"))?,
        )
        .map_err(|_e| tera::Error::call_filter("format_date", "unkown"))?
    } else if offset.whole_weeks() != 0 {
        format!(
            "{} weeks, {} days",
            offset.whole_weeks(),
            offset.whole_days() % 7
        )
    } else {
        format!(
            "{} days, {} hours",
            offset.whole_days(),
            offset.whole_hours() % 24
        )
    }))
}

fn prepend_line_break(value: &Value, data: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    if !data.is_empty() {
        return Err(tera::Error::msg(
            "The format string filter takes no parameters",
        ));
    }
    match value {
        Value::String(s) => Ok(Value::String(format!("\n{s}"))),
        _ => Err(tera::Error::msg("prepend_line_break: Expected a string")),
    }
}

fn empty(value: Option<&Value>, args: &[Value]) -> tera::Result<bool> {
    if !args.is_empty() {
        return Err(tera::Error::msg(
            "The format string filter takes no parameters",
        ));
    }

    Ok(match value {
        Some(e) => match e {
            Value::Null => false,
            Value::Bool(_) => false,
            Value::Number(_number) => false,
            Value::String(e) => e.is_empty(),
            Value::Array(values) => values.is_empty(),
            Value::Object(map) => map.is_empty(),
        },
        None => false,
    })
}
