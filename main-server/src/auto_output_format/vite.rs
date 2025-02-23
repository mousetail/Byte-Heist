use std::{collections::HashMap, sync::OnceLock};

use serde::Deserialize;
use tera::{escape_html, Value};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestEntry {
    file: String,
    #[serde(default)]
    #[allow(unused)]
    name: Option<String>,
    #[serde(default)]
    #[allow(unused)]
    is_entry: bool,
    #[serde(default)]
    css: Vec<String>,
    #[serde(default)]
    imports: Vec<String>,
    #[allow(unused)]
    #[serde(default)]
    dynamic_imports: Vec<String>,
}

struct Imports<'a> {
    scripts: Vec<&'a str>,
    styles: Vec<&'a str>,
}

impl ManifestEntry {
    fn find_all_imports(mut files: Vec<&str>) -> Imports {
        use std::fs::OpenOptions;

        let manifest = VITE_MANIFEST.get_or_init(|| {
            let file = OpenOptions::new()
                .read(true)
                .open("static/target/.vite/manifest.json")
                .expect("Failed to open vite manifest. Did you run `vite build`?");

            serde_json::from_reader(file).unwrap()
        });

        let mut scripts = vec![];
        let mut styles = vec![];
        while let Some(file) = files.pop() {
            let value = manifest.get(file).unwrap();
            scripts.push(value.file.as_str());
            styles.extend(value.css.iter().map(|k| k.as_str()));

            for import in &value.imports {
                files.push(import);
            }
        }

        Imports { scripts, styles }
    }
}

static VITE_MANIFEST: OnceLock<HashMap<String, ManifestEntry>> = OnceLock::new();

pub fn load_assets(values: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    let scripts = values
        .get("modules")
        .ok_or_else(|| tera::Error::msg("Expected argument \"modules\'"))?;
    let modules = match scripts {
        Value::Array(arr) => arr
            .iter()
            .map(|k| k.as_str())
            .collect::<Option<Vec<&str>>>()
            .ok_or_else(|| tera::Error::msg("Expected modules to be an array of strings"))?,
        Value::String(k) => vec![k.as_str()],
        _ => return Err(tera::Error::msg("Expected scripts to be a string or array")),
    };

    if cfg!(debug_assertions) {
        let mut out: String =
            r#"<script type="module" src="http://localhost:5173/static/target/@vite/client"></script>"#
                .to_string();
        for module in modules {
            out.push_str(&format!(
                r#"<script type="module" src="http://localhost:5173/static/target/{}"></script>"#,
                escape_html(module)
            ));
        }

        Ok(Value::String(out))
    } else {
        let imports = ManifestEntry::find_all_imports(modules);
        Ok(Value::String(
            imports
                .scripts
                .into_iter()
                .map(|script| {
                    format!(
                        r#"<script type="module" src="/static/target/{}"></script>"#,
                        escape_html(script)
                    )
                })
                .chain(imports.styles.into_iter().map(|style| {
                    format!(
                        r#"<link rel="stylesheet" href="/static/target/{}">"#,
                        escape_html(style)
                    )
                }))
                .collect(),
        ))
    }
}
