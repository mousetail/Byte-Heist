use std::{path::PathBuf, process::Stdio, sync::Arc};

use async_process::Command;
use common::langs::{LANGS, Lang};

use crate::{cachemap::CacheMap, error::RunProcessError};

pub async fn install_plugin(lang: &'static Lang) -> Result<CacheMap<String, ()>, RunProcessError> {
    println!("Installing language version {}", lang.display_name);
    let plugin_install_output = Command::new("asdf")
        .args(["plugin", "add", lang.plugin_name, lang.plugin])
        .stderr(Stdio::inherit())
        .status()
        .await?;
    if !plugin_install_output.success() {
        return Err(RunProcessError::NonZeroStatusCode(
            plugin_install_output.code(),
        ));
    }
    Ok(CacheMap::new())
}

pub async fn install_language_version(
    lang: &'static Lang,
    version: &str,
) -> Result<(), RunProcessError> {
    println!(
        "Installing language version {} {}",
        lang.display_name, version
    );
    let mut command = Command::new("asdf");
    command
        .args(["install", lang.plugin_name, version])
        .stderr(Stdio::inherit());

    for env in lang.install_env {
        command.env(env.0, env.1);
    }

    let status = command.status().await?;

    if !status.success() {
        return Err(RunProcessError::NonZeroStatusCode(status.code()));
    }
    Ok(())
}

pub async fn install_lang(
    lang_name: String,
    version: &str,
    versions: Arc<CacheMap<String, CacheMap<String, ()>>>,
) -> Result<(), RunProcessError> {
    let version = version.to_owned();
    tokio::spawn(async move {
        let lang = match LANGS.get(&lang_name) {
            Some(e) => e,
            None => panic!("Unexpected lang {lang_name}"),
        };

        let lang_version_token = versions.get(lang.plugin_name.to_owned());
        let lang_versions = lang_version_token
            .get_or_try_init(|| install_plugin(lang))
            .await?;

        let specific_version_token = lang_versions.get(version.to_owned());

        let _specific_version = specific_version_token
            .get_or_try_init(|| install_language_version(lang, &version))
            .await?;

        Ok(())
    })
    .await
    .map_err(|_| RunProcessError::JoinFail)?
}

pub async fn get_lang_directory(lang: &Lang, version: &str) -> Result<PathBuf, RunProcessError> {
    let lang_folder = Command::new("asdf")
        .args(["where", lang.plugin_name, version])
        .stderr(Stdio::inherit())
        .output()
        .await?;
    if !lang_folder.status.success() {
        return Err(RunProcessError::NonZeroStatusCode(
            lang_folder.status.code(),
        ));
    }

    let buff = PathBuf::from(String::from_utf8(lang_folder.stdout).unwrap().trim());
    Ok(buff)
}
