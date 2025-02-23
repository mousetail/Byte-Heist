use common::{JudgeResult, TestCase};
use futures_util::AsyncReadExt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FinalVerdict {
    pass: bool,
}

const MAX_TEST_CASES: usize = 50;
const MAX_OUTPUT_LENGTH: usize = 10000;

pub fn apply_to_judge_result(judge_result: &mut JudgeResult, line: &[u8]) {
    if line.is_empty() {
        return;
    }
    if judge_result.test_cases.len() > MAX_TEST_CASES {
        judge_result.pass = false;
        eprintln!("Maximum number of test cases exceeded");
        return;
    }
    match serde_json::from_slice::<TestCase>(line) {
        Ok(mut test_case) => {
            test_case.truncate(MAX_OUTPUT_LENGTH);
            judge_result.test_cases.push(test_case);
        }
        Err(_e) => match serde_json::from_slice::<FinalVerdict>(line) {
            Ok(FinalVerdict { pass: new_pass }) => judge_result.pass = new_pass,
            Err(_e) => judge_result.test_cases.push(TestCase {
                name: Some("Judge Debug Message".to_owned()),
                pass: common::TestPassState::Info,
                result_display: common::ResultDisplay::Text(
                    String::from_utf8_lossy(line).to_string(),
                ),
            }),
        },
    }
}

pub async fn parse_judge_result_from_stream(mut stream: impl AsyncReadExt + Unpin) -> JudgeResult {
    let mut judge_result = JudgeResult {
        test_cases: vec![],
        pass: false,
    };

    let mut line_buffer = vec![];
    loop {
        let mut buffer = [0; 512];
        let value = stream.read(&mut buffer).await.unwrap();
        if value == 0 {
            break;
        }

        let mut part = &buffer[..value];
        while let Some(i) = part.iter().position(|&d| d == b'\n') {
            line_buffer.extend_from_slice(&part[..i]);

            apply_to_judge_result(&mut judge_result, &line_buffer);
            line_buffer.clear();

            part = &part[i + 1..];
        }
        line_buffer.extend_from_slice(part);
    }
    judge_result
}
