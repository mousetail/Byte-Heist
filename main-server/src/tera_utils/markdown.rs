use std::cell::OnceCell;

use markdown_it::{
    MarkdownIt,
    plugins::{cmark::inline::link::Link, extra::syntect::set_theme},
};
use tera::Filter;

use super::syntax_highlighting::SYNTECT_THEME;

thread_local! {
    static MARKDOWN: OnceCell<MarkdownIt> = const { OnceCell::new() };
}

pub fn render_markdown(source: &str) -> String {
    MARKDOWN.with(|markdown| {
        let parser = markdown.get_or_init(|| {
            let mut parser = markdown_it::MarkdownIt::new();
            markdown_it::plugins::cmark::add(&mut parser);
            markdown_it::plugins::extra::add(&mut parser);

            // todo: Adjust this if we ever need a light mode
            set_theme(&mut parser, SYNTECT_THEME);

            parser
        });

        let mut ast = parser.parse(source);

        // TODO: this is probbly better with a plugin
        ast.walk_mut(|e, _index| {
            if e.is::<Link>() {
                e.attrs.push(("rel", "nofollow noopener".to_owned()));
            }
        });

        format!("<div class=\"markdown\">{}</div>", ast.render())
    })
}

pub struct MarkdownFilter;

impl Filter for MarkdownFilter {
    fn is_safe(&self) -> bool {
        true
    }

    fn filter(
        &self,
        value: &tera::Value,
        args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        if !args.is_empty() {
            return Err(tera::Error::msg("The markdown function takes no arguments"));
        }
        let text = value
            .as_str()
            .ok_or_else(|| tera::Error::msg("Expected type to be a string"))?;
        Ok(tera::Value::String(render_markdown(text)))
    }
}
