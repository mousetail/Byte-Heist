use std::sync::Arc;

use common::RunLangOutput;
use tokio::process::Command;

use crate::{
    Message, cachemap::CacheMap, error::RunLangError, install_lang::install_lang,
    judge::run_lang_with_judge,
};

const MAX_CONCURRENT_RUNS: usize = 4;
static RUNS_SEMAPHORE: tokio::sync::Semaphore =
    tokio::sync::Semaphore::const_new(MAX_CONCURRENT_RUNS);

pub async fn process_message(
    message: Message,
    lang_versions: Arc<CacheMap<String, CacheMap<String, ()>>>,
) -> Result<RunLangOutput, RunLangError> {
    install_lang(message.lang.clone(), &message.version, lang_versions)
        .await
        .map_err(RunLangError::PluginInstallFailure)?;

    let _semaphore = RUNS_SEMAPHORE
        .acquire()
        .await
        .map_err(RunLangError::SemaphoreError)?;
    let output = run_lang_with_judge(
        &message.lang,
        &message.version,
        &message.code,
        &message.judge,
    )
    .await?;
    Ok(output)
}

async fn get_versions_for_language(line: &str) -> (String, CacheMap<String, ()>) {
    let parts = line.split_ascii_whitespace().collect::<Vec<_>>();
    let Some(name) = parts.first() else {
        panic!("bad output from asdf plugin list: {line} {parts:?}")
    };

    let versions = Command::new("asdf")
        .args(["list", name])
        .output()
        .await
        .unwrap();

    if !versions.status.success() {
        println!("Finding versions failed");
    }

    (
        (*name).to_owned(),
        String::from_utf8(versions.stdout)
            .unwrap()
            .lines()
            .map(|k| (k.trim().to_owned(), ()))
            .collect::<CacheMap<_, ()>>(),
    )
}

pub async fn get_lang_versions() -> CacheMap<String, CacheMap<String, ()>> {
    let output = Command::new("asdf")
        .args(["plugin", "list"])
        .output()
        .await
        .unwrap();
    if !output.status.success() {
        panic!("Finding the list of plugins failed");
    }

    let output_text = String::from_utf8(output.stdout).unwrap();
    futures_util::future::join_all(output_text.lines().map(get_versions_for_language))
        .await
        .into_iter()
        .collect()
}
