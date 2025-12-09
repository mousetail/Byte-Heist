use std::{
    process::Stdio,
    time::{Duration, Instant},
};

use common::{JudgeResult, RunLangOutput, TestCase, Timers, langs::LANGS};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::Command,
};

use crate::{error::RunLangError, run::RunLangContext, stopwatch::start_stopwatch};

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

    let start_time = Instant::now();

    let (mut sender, receiver) = tokio::sync::mpsc::channel(16);
    let (output, timers) = start_stopwatch(
        Timers {
            judge: Duration::from_secs(1) + lang.extra_runtime.judge,
            run: Duration::from_secs(TIMEOUT) + lang.extra_runtime.run,
            compile: Duration::from_secs(1) + lang.extra_runtime.compile,
        },
        receiver,
        Box::pin(async move {
            let mut judge_result = JudgeResult {
                pass: false,
                test_cases: vec![],
            };

            while let Some(line) = lines.next_line().await? {
                let data: JudgeResponse = serde_json::from_str(&line).map_err(|e| {
                    RunLangError::RunLang(crate::error::RunProcessError::SerializationFailed(e))
                })?;
                match data {
                    JudgeResponse::RunRequest(run_request) => {
                        let result = context
                            .run(&run_request.code, run_request.input.as_deref(), &mut sender)
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
        }),
    )
    .await;

    let end_time = Instant::now();

    let timed_out = output.is_none();
    let judge_result = output.unwrap_or(Ok(JudgeResult {
        pass: false,
        test_cases: vec![],
    }))?;

    let mut stderr = command
        .stderr
        .take()
        .expect("Expected the child to have a stderr");

    let mut error = String::new();
    stderr.read_to_string(&mut error).await?;

    command.kill().await?;

    Ok(RunLangOutput {
        tests: judge_result,
        stderr: error,
        timed_out,
        runtime: (end_time - start_time).as_secs_f32(),
        timers,
    })
}
