use std::str::FromStr;
use std::{collections::HashMap, sync::OnceLock};

use axum::response::{IntoResponse, Response};
use common::AchievementType;
use common::langs::LANGS;
use sqlx::types::time::OffsetDateTime;
use tera::{Filter, Tera, Value, to_value};
use tower_sessions::cookie::time::macros::format_description;

use crate::tera_utils::markdown::MarkdownFilterWithTableOfContents;
use crate::tera_utils::syntax_highlighting::SyntaxHighight;

use super::markdown::MarkdownFilter;

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
        Tera::new("templates/**/*").map(|mut tera| {
            tera.autoescape_on(vec![".html.jinja", ".xml.jinja", ".html", ".xml"]);
            tera.register_function("languages", get_langs);
            tera.register_function("modules", load_assets);
            tera.register_filter("markdown", MarkdownFilter);
            tera.register_filter(
                "markdown_with_table_of_contents",
                MarkdownFilterWithTableOfContents,
            );
            tera.register_filter("prepend_linebreak", prepend_line_break);
            tera.register_filter("format_number", format_number);
            tera.register_filter("format_date", format_date);
            tera.register_tester("empty", empty);
            tera.register_filter("syntax_highlight", SyntaxHighight);
            tera.register_filter(
                "get_achievement_icon",
                MappingStringToStringFilter {
                    f: |e| {
                        AchievementType::from_str(e)
                            .map(|k| k.get_icon())
                            .unwrap_or_default()
                    },
                },
            );
            tera.register_filter(
                "get_achievement_name",
                MappingStringToStringFilter {
                    f: |e| {
                        AchievementType::from_str(e)
                            .map(|k| k.get_achievement_name())
                            .unwrap_or_default()
                            .to_owned()
                    },
                },
            );
            tera.register_filter(
                "get_achievement_description",
                MappingStringToStringFilter {
                    f: |e| {
                        AchievementType::from_str(e)
                            .map(|k| k.get_achievement_description())
                            .unwrap_or_default()
                            .to_owned()
                    },
                },
            );
            tera.register_filter(
                "un_camelcase",
                MappingStringToStringFilter {
                    f: |e| {
                        e.chars()
                            .flat_map(|k| {
                                k.is_ascii_uppercase().then_some(' ').into_iter().chain([k])
                            })
                            .collect::<String>()
                    },
                },
            );
            tera.register_filter(
                "language_display_name",
                MappingStringToStringFilter { f: get_lang_name },
            );
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

fn get_lang_name(e: &str) -> String {
    LANGS
        .get(e)
        .map(|e| e.display_name)
        .unwrap_or(e)
        .to_string()
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
    let include_day = match data.get("include_day") {
        None => true,
        Some(Value::Bool(e)) => *e,
        _ => return Err(tera::Error::msg("include_day must be a boolean")),
    };
    if !data.is_empty() && (data.len() != 1 || !data.contains_key("include_day")) {
        return Err(tera::Error::msg(
            "The format string filter takes one paramter: include day",
        ));
    }
    let date: OffsetDateTime = serde_json::from_value(value.clone()).map_err(tera::Error::json)?;

    let offset = date - OffsetDateTime::now_utc();
    let (relative_time_prefix, relative_time_postfix) = if offset.is_positive() {
        ("in ", "")
    } else {
        ("", " ago")
    };
    let offset = offset.abs();

    Ok(Value::String(if offset.whole_weeks() > 12 {
        date.format(if include_day {
            format_description!("[day] [month repr:long] [year]")
        } else {
            format_description!("[month repr:long] [year]")
        })
        .map_err(|_e| tera::Error::call_filter("format_date", "unkown"))?
    } else if offset.whole_hours() == 0 {
        format!("{relative_time_prefix}a few minutes{relative_time_postfix}")
    } else if offset.whole_weeks() != 0 {
        format!(
            "{relative_time_prefix}{} weeks, {} days{relative_time_postfix}",
            offset.whole_weeks(),
            offset.whole_days() % 7
        )
    } else {
        format!(
            "{relative_time_prefix}{} days, {} hours{relative_time_postfix}",
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

struct MappingStringToStringFilter<F: Fn(&str) -> String> {
    f: F,
}

impl<F: Fn(&str) -> String + Send + Sync> Filter for MappingStringToStringFilter<F> {
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
        if !args.is_empty() {
            return Err(tera::Error::msg("This filter takes no arguments"));
        }

        let data = match value {
            tera::Value::String(e) => (self.f)(e),
            _ => return Err(tera::Error::msg("This filter expects a string")),
        };

        Ok(tera::Value::String(data))
    }
}
