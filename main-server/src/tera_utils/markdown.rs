use std::cell::OnceCell;

use common::slug::Slug;
use markdown_it::{
    MarkdownIt, Node,
    plugins::{
        cmark::{block::heading::ATXHeading, inline::link::Link},
        extra::syntect::set_theme,
    },
};
use tera::{Filter, Map, Number};

use super::syntax_highlighting::SYNTECT_THEME;

thread_local! {
    static MARKDOWN: OnceCell<MarkdownIt> = const { OnceCell::new() };
}

fn render_markdown<F>(source: &str, f: F) -> String
where
    F: FnMut(&mut Node, u32),
{
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

        let mut headers = vec![];

        // TODO: this is probbly better with a plugin
        ast.walk_mut(|e, _index| {
            if e.is::<Link>() {
                e.attrs.push(("rel", "nofollow noopener".to_owned()));
            }
            if e.is::<ATXHeading>() {
                let slug = format!("{}", Slug(&e.collect_text()));

                e.attrs.push(("id", slug.clone()));
                headers.push(slug);
            }
        });
        ast.walk_mut(f);

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
        Ok(tera::Value::String(render_markdown(text, |_, _| ())))
    }
}

pub struct MarkdownFilterWithTableOfContents;

impl Filter for MarkdownFilterWithTableOfContents {
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

        let mut headings = vec![];

        let content = render_markdown(text, |node, _index| {
            if let Some(e) = node.cast::<ATXHeading>() {
                headings.push((e.level, node.collect_text()))
            }
        });

        let mut map = Map::new();
        map.insert("content".to_string(), tera::Value::String(content));
        map.insert(
            "headings".to_string(),
            tera::Value::Array(
                headings
                    .into_iter()
                    .map(|i| {
                        let mut map = Map::new();
                        map.insert("text".to_owned(), tera::Value::String(i.1));
                        map.insert(
                            "level".to_owned(),
                            tera::Value::Number(
                                Number::from_i128(i.0 as i128).expect("Surely a u8 fits in a i128"),
                            ),
                        );
                        tera::Value::Object(map)
                    })
                    .collect(),
            ),
        );
        Ok(tera::Value::Object(map))
    }
}
