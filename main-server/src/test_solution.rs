use std::{borrow::Cow, time::Duration};

use common::RunLangOutput;
use serde::Serialize;

use crate::error::Error;

#[derive(Serialize)]
struct TestRunnerRequest<'a> {
    lang: &'a str,
    version: &'a str,
    code: &'a str,
    judge: &'a str,
}

pub async fn test_solution(
    code: &str,
    language: &str,
    version: &str,
    judge: &str,
) -> Result<RunLangOutput, Error> {
    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:3000")
        .json(&TestRunnerRequest {
            lang: language,
            version,
            code,
            judge,
        })
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                Error::RunLang(Cow::Borrowed(concat!(
                    "Timeout connecting to the lang runner, this usually means the language ",
                    "is in the process of being installed"
                )))
            } else {
                Error::RunLang(Cow::Borrowed("Failed to connect to the lang runner"))
            }
        })?;

    if !resp.status().is_success() {
        return Err(Error::RunLang(Cow::Owned(
            resp.text().await.map_err(|_| Error::ServerError)?,
        )));
    }

    let out = resp
        .json::<RunLangOutput>()
        .await
        .map_err(|_| Error::RunLang(Cow::Borrowed("Failed to parse json")))?;

    Ok(out)
}
