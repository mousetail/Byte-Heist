use std::cell::OnceCell;

use syntect::html::highlighted_html_for_string;
use tera::Filter;

pub const SYNTECT_THEME: &str = "base16-eighties.dark";

thread_local! {
    static THEME: OnceCell<(syntect::highlighting::Theme, syntect::parsing::SyntaxSet)> = const { OnceCell::new() };
}

pub struct SyntaxHighight;

impl Filter for SyntaxHighight {
    fn is_safe(&self) -> bool {
        true
    }

    fn filter(
        &self,
        value: &tera::Value,
        args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        if args.len() != 1 {
            return Err(tera::Error::msg(
                "The syntaxHighlight expects exactly one argument",
            ));
        }
        let lang = match args.get("lang") {
            Some(tera::Value::String(e)) => e,
            Some(_) => {
                return Err(tera::Error::msg(
                    "The syntaxHighlight filter expects lang to be of type string",
                ))
            }
            None => {
                return Err(tera::Error::msg(
                    "The syntaxHighlight expects one argument: lang",
                ))
            }
        };

        let code = match value {
            tera::Value::String(e) => e,
            _ => return Err(tera::Error::msg("Expected input of kind string")),
        };

        THEME.with(|theme| -> Result<tera::Value, tera::Error> {
            let (theme, syntax_set) = theme.get_or_init(|| {
                let themes = syntect::highlighting::ThemeSet::load_defaults();
                let syntax_set = syntect::parsing::SyntaxSet::load_defaults_newlines();

                (themes.themes[SYNTECT_THEME].clone(), syntax_set)
            });

            let mut html = highlighted_html_for_string(
                code,
                syntax_set,
                syntax_set.find_syntax_by_token(lang).ok_or_else(|| {
                    tera::Error::msg(format!("Unknown syntax highlighting language: {lang}"))
                })?,
                theme,
            )
            .map_err(|e| tera::Error::msg(format!("error: {e:?}")))?;

            html.insert_str(4, " class=\"code-pre\"");

            Ok(tera::Value::String(html))
        })
    }
}
