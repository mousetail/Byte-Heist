use std::{
    process::Stdio,
    time::{Duration, Instant},
};

use common::{JudgeResult, RunLangOutput, TestCase, TimerType, Timers, langs::LANGS};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{ChildStdin, Command},
    sync::mpsc::Sender,
};

use crate::{error::RunLangError, run::RunLangContext, stopwatch::start_stopwatch};

const MAX_TEST_CASES: usize = 50;
const TIMEOUT: u64 = 3;
const MAX_CODE_SIZE: usize = 64 * 1024;

#[derive(Deserialize, Debug)]
struct FinalVerdict {
    pass: bool,
    points: Option<i32>,
}

#[derive(Serialize)]
struct RunnerInput<'a> {
    lang: &'a str,
    code: &'a str,
    judge: &'a str,
    max_code_size: usize,
    max_input_size: usize,
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

async fn handle_judge_command(
    data: JudgeResponse,
    judge_result_ref: &mut JudgeResult,
    sender: &mut Sender<TimerType>,
    stdin: &mut ChildStdin,
    context: &mut RunLangContext,
) -> Result<(), RunLangError> {
    match data {
        JudgeResponse::RunRequest(run_request) => {
            if run_request.code.len() > MAX_CODE_SIZE {
                stdin
                    .write_all(
                        &serde_json::to_vec(&crate::error::RunProcessError::CodeTooLarge).map_err(
                            |e| {
                                RunLangError::RunLang(
                                    crate::error::RunProcessError::SerializationFailed(e),
                                )
                            },
                        )?,
                    )
                    .await?;
                return Ok(());
            }

            if run_request
                .input
                .as_deref()
                .is_some_and(|i| i.len() > MAX_CODE_SIZE)
            {
                stdin
                    .write_all(
                        &serde_json::to_vec(&crate::error::RunProcessError::InputTooLarge)
                            .map_err(|e| {
                                RunLangError::RunLang(
                                    crate::error::RunProcessError::SerializationFailed(e),
                                )
                            })?,
                    )
                    .await?;
                return Ok(());
            }

            let result = context
                .run(&run_request.code, run_request.input.as_deref(), sender)
                .await
                .map_err(RunLangError::RunLang)?;

            stdin
                .write_all(&serde_json::to_vec(&result).map_err(|e| {
                    RunLangError::RunLang(crate::error::RunProcessError::SerializationFailed(e))
                })?)
                .await?;
        }
        JudgeResponse::TestCase(test_case) => {
            judge_result_ref.test_cases.push(test_case);

            if judge_result_ref.test_cases.len() > MAX_TEST_CASES {
                Err(RunLangError::MaxTestCasesExceeded)?;
            }
        }
        JudgeResponse::FinalVerdict(final_verdict) => {
            println!("final_verdict: {final_verdict:?}");
            judge_result_ref.pass = final_verdict.pass;
            judge_result_ref.points = final_verdict.points;
        }
    }

    Ok(())
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
        max_code_size: MAX_CODE_SIZE,
        max_input_size: MAX_CODE_SIZE,
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

    let mut judge_result = JudgeResult {
        pass: false,
        test_cases: vec![],
        points: None,
    };
    let judge_result_ref = &mut judge_result;
    let (out, timers) = start_stopwatch(
        Timers {
            judge: Duration::from_secs(1) + lang.extra_runtime.judge,
            run: Duration::from_secs(TIMEOUT) + lang.extra_runtime.run,
            compile: Duration::from_secs(1) + lang.extra_runtime.compile,
        },
        receiver,
        Box::pin(async move {
            while let Some(line) = lines.next_line().await? {
                let data: JudgeResponse = serde_json::from_str(&line).map_err(|e| {
                    RunLangError::RunLang(crate::error::RunProcessError::SerializationFailed(e))
                })?;
                handle_judge_command(
                    data,
                    judge_result_ref,
                    &mut sender,
                    &mut stdin,
                    &mut context,
                )
                .await?;
            }

            Ok::<(), RunLangError>(())
        }),
    )
    .await;

    let end_time = Instant::now();
    let timed_out = out.is_none();
    if timed_out {
        judge_result.pass = false;
    }

    command.kill().await?;

    let mut stderr = command
        .stderr
        .take()
        .expect("Expected the child to have a stderr");

    let mut error = String::new();
    stderr.read_to_string(&mut error).await?;

    Ok(RunLangOutput {
        tests: judge_result,
        stderr: error,
        timed_out,
        runtime: (end_time - start_time).as_secs_f32(),
        timers,
    })
}
