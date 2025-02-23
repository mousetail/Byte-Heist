use std::time::Duration;

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
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .map_err(|_e| Error::RunLang("Failed to connect to the lang runner".to_string()))?;

    if !resp.status().is_success() {
        return Err(Error::RunLang(
            resp.text().await.map_err(|_| Error::ServerError)?,
        ));
    }

    let out = resp
        .json::<RunLangOutput>()
        .await
        .map_err(|_| Error::RunLang("Failed to parse json".to_string()))?;

    Ok(out)
}
