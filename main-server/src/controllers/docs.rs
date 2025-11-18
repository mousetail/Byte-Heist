use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    fs::read_to_string,
    io::ErrorKind,
    ops::Deref,
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    sync::LazyLock,
};

use axum::extract::Path;
use serde::Serialize;

use crate::error::Error;

#[derive(Serialize, Clone)]
struct File {
    path: String,
    display_name: String,
}

const FILES: LazyLock<std::io::Result<Vec<File>>> = LazyLock::new(|| {
    std::fs::read_dir("doc").map(|i| {
        i.flatten()
            .map(|d| d.file_name().into_string())
            .flatten()
            .map(|k| File {
                display_name: k
                    .split('/')
                    .last()
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
    prev: Option<String>,
    next: Option<String>,
    canonical_path: String,
    display_name: String,
    files: Vec<File>,
}

pub async fn get_doc(path: Option<Path<String>>) -> Result<GetDocOutput, Error> {
    let files = FILES;
    let files = files.as_ref().map_err(|_| Error::ServerError)?;

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
        Some(
            files[(position + files.len() - 1) % files.len()]
                .path
                .clone(),
        ),
        Some(files[(position + 1) % files.len()].path.clone()),
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
        prev: prev.to_owned(),
        next: next.to_owned(),
        canonical_path: files[position].path.clone(),
        display_name: files[position].display_name.clone(),
        files: files.clone(),
    })
}
