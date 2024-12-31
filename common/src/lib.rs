pub mod langs;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JudgeResult {
    pub pass: bool,
    pub test_cases: Vec<TestCase>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunLangOutput {
    pub tests: JudgeResult,
    pub stderr: String,
    pub timed_out: bool,
}

#[derive(Serialize, Deserialize)]
pub enum TestPassState {
    /// The test passed
    Pass,
    /// The test failed and caused the entire challenge to fail
    Fail,
    /// This particular test is only informational and has no effect on the pass/fail of the entire challenge
    Info,
    /// This test failed but failure does not nececairly mean the entire challenge failed. Can be used if, for example, you only need 5 out of 6
    /// Tests to pass
    Warning,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCase {
    #[serde(default)]
    pub name: Option<String>,
    pub pass: TestPassState,
    pub result_display: ResultDisplay,
}

impl TestCase {
    pub fn truncate(&mut self, length: usize) {
        self.result_display.truncate(length);
    }
}

fn create_default_sep() -> String {
    "\n".to_string()
}

#[derive(Serialize, Deserialize)]
pub enum ResultDisplay {
    Empty,
    Text(String),
    Diff {
        #[serde(default)]
        input: Option<String>,
        output: String,
        expected: String,
        #[serde(default = "create_default_sep")]
        sep: String,
    },
    Run {
        #[serde(default)]
        input: Option<String>,
        output: String,
        error: String,
    },
}

impl ResultDisplay {
    pub fn truncate(&mut self, length: usize) {
        match self {
            ResultDisplay::Empty => {}
            ResultDisplay::Text(e) => e.truncate(length),
            ResultDisplay::Diff {
                output,
                expected,
                sep,
                input,
            } => {
                output.truncate(length);
                expected.truncate(length);
                if let Some(d) = input.as_mut() {
                    d.truncate(length)
                }
                sep.truncate(5);
            }
            ResultDisplay::Run {
                input,
                output,
                error,
            } => {
                if let Some(input) = input {
                    input.truncate(length);
                }
                output.truncate(length);
                error.truncate(length);
            }
        }
    }
}
