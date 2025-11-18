use std::{borrow::Cow, fs::read_to_string, io::ErrorKind, path::PathBuf, sync::LazyLock};

use axum::extract::Path;
use serde::Serialize;

use crate::error::Error;

#[derive(Serialize, Clone)]
struct File {
    path: String,
    display_name: String,
}

static FILES: LazyLock<std::io::Result<Vec<File>>> = LazyLock::new(|| {
    std::fs::read_dir("doc").map(|i| {
        i.flatten()
            .flat_map(|d| d.file_name().into_string())
            .map(|k| File {
                display_name: k
                    .split('/')
                    .next_back()
                    .and_then(|d| d.split_once('.'))
                    .map(|z| z.0.replace('_', " "))
                    .unwrap_or_default(),
                path: k,
            })
            .collect()
    })
});

#[derive(Serialize)]
pub struct GetDocOutput {
    content: String,
    prev: &'static str,
    next: &'static str,
    canonical_path: &'static str,
    display_name: &'static str,
    files: &'static [File],
}

pub async fn get_doc(path: Option<Path<String>>) -> Result<GetDocOutput, Error> {
    let files = FILES.as_ref().map_err(|_| Error::ServerError)?;

    let original_path = path
        .map(|Path(e)| e)
        .unwrap_or("intro_to_code_golf.md".into());

    if original_path.contains("..") {
        return Err(Error::BadRequest("Path must not contain .."));
    }

    let mut transformed_path = original_path.trim_start_matches("/").to_owned();
    if !transformed_path.ends_with(".md") {
        transformed_path.push_str(".md");
    }

    let Some(position) = files
        .iter()
        .position(|e| e.path.eq_ignore_ascii_case(&transformed_path))
    else {
        return Err(Error::NotFound);
    };

    if files[position].path != original_path {
        return Err(Error::Redirect(Cow::Owned(format!(
            "/doc/{}",
            files[position].path,
        ))));
    }

    let (prev, next) = (
        &files[(position + files.len() - 1) % files.len()].path,
        &files[(position + 1) % files.len()].path,
    );

    let mut root = PathBuf::new();
    root.push("doc");
    let joined = root.join(&files[position].path);

    let s = read_to_string(&joined).map_err(|e| {
        if e.kind() == ErrorKind::NotFound {
            Error::NotFound
        } else {
            eprintln!("{joined:?}");
            Error::ServerError
        }
    })?;

    Ok(GetDocOutput {
        content: s,
        prev,
        next,
        canonical_path: &files[position].path,
        display_name: &files[position].display_name,
        files: files.as_slice(),
    })
}
