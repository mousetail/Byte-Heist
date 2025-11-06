use std::{process::Stdio, time::Duration};

use async_process::Command;
use common::{JudgeResult, RunLangOutput, TestCase, langs::LANGS};
use futures_util::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, StreamExt, io::BufReader};
use serde::{Deserialize, Serialize};

use crate::{error::RunLangError, run::RunLangContext};

const MAX_TEST_CASES: usize = 50;
const TIMEOUT: u64 = 3;

#[derive(Deserialize, Debug)]
struct FinalVerdict {
    pass: bool,
}

#[derive(Serialize)]
struct RunnerInput<'a> {
    lang: &'a str,
    code: &'a str,
    judge: &'a str,
}

#[derive(Deserialize, Debug)]
struct RunRequest {
    code: String,
    input: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum JudgeResponse {
    RunRequest(RunRequest),
    TestCase(TestCase),
    FinalVerdict(FinalVerdict),
}

pub async fn run_lang_with_judge(
    language: &str,
    version: &str,
    code: &str,
    judge: &str,
) -> Result<RunLangOutput, RunLangError> {
    let lang = LANGS.get(language).ok_or(RunLangError::RunLang(
        crate::error::RunProcessError::NoSuchLanguage,
    ))?;

    let data = serde_json::to_string(&RunnerInput {
        lang: language,
        code,
        judge,
    })
    .map_err(|e| RunLangError::RunLang(crate::error::RunProcessError::SerializationFailed(e)))?;

    let mut command = Command::new("/usr/local/bin/deno")
        .args([
            "run",
            "--allow-read=./scripts/runner-lib.ts,./scripts/words.txt",
            "scripts/runner.ts",
        ])
        .arg(data)
        .env("NO_COLOR", "TRUE")
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(RunLangError::IOError)?;

    let mut stdin = command.stdin.take().expect("Command should have stdin");
    let mut lines =
        BufReader::new(command.stdout.take().expect("Command should have stdout")).lines();

    let mut context = RunLangContext::new(language, version)
        .await
        .map_err(RunLangError::RunLang)?;

    let result = tokio::time::timeout(
        Duration::from_secs(TIMEOUT + lang.extra_runtime),
        async move {
            let mut judge_result = JudgeResult {
                pass: false,
                test_cases: vec![],
            };

            while let Some(line) = lines.next().await {
                let line = line?;
                let data: JudgeResponse = serde_json::from_str(&line).map_err(|e| {
                    RunLangError::RunLang(crate::error::RunProcessError::SerializationFailed(e))
                })?;
                match data {
                    JudgeResponse::RunRequest(run_request) => {
                        let result = context
                            .run(&run_request.code, run_request.input.as_deref())
                            .await
                            .map_err(RunLangError::RunLang)?;

                        stdin
                            .write_all(&serde_json::to_vec(&result).map_err(|e| {
                                RunLangError::RunLang(
                                    crate::error::RunProcessError::SerializationFailed(e),
                                )
                            })?)
                            .await?;
                    }
                    JudgeResponse::TestCase(test_case) => {
                        judge_result.test_cases.push(test_case);

                        if judge_result.test_cases.len() > MAX_TEST_CASES {
                            Err(RunLangError::MaxTestCasesExceeded)?;
                        }
                    }
                    JudgeResponse::FinalVerdict(final_verdict) => {
                        judge_result.pass = final_verdict.pass
                    }
                }
            }

            Ok::<JudgeResult, RunLangError>(judge_result)
        },
    )
    .await;

    let (jude_result, timed_out) = match result {
        Ok(e) => (e?, false),
        Err(_) => (
            JudgeResult {
                pass: false,
                test_cases: vec![],
            },
            true,
        ),
    };

    let mut stderr = command
        .stderr
        .take()
        .expect("Expected the child to have a stderr");

    let mut error = String::new();
    stderr.read_to_string(&mut error).await?;

    command.kill()?;
    command.status().await?;

    Ok(RunLangOutput {
        tests: jude_result,
        stderr: error,
        //format!("Output:\n{stdout}\nError:\n{stderr}"),
        timed_out,
        runtime: 0.0,
    })
}
