pub mod achievements;
pub mod langs;
pub mod slug;
pub mod urls;

use std::time::Duration;

pub use achievements::{AchievementCategory, AchievementType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JudgeResult {
    pub pass: bool,
    pub test_cases: Vec<TestCase>,
}

#[derive(Clone, Copy)]
pub enum TimerType {
    Run,
    Compile,
    Judge,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default)]
pub struct Timers {
    pub run: Duration,
    pub compile: Duration,
    pub judge: Duration,
}

pub static DEFAULT_TIMERS: Timers = Timers {
    run: Duration::ZERO,
    compile: Duration::ZERO,
    judge: Duration::ZERO,
};

impl Timers {
    pub fn get_mut_type(&mut self, timer_type: TimerType) -> &mut Duration {
        match timer_type {
            TimerType::Run => &mut self.run,
            TimerType::Compile => &mut self.compile,
            TimerType::Judge => &mut self.judge,
        }
    }

    pub fn get_type(&self, timer_type: TimerType) -> &Duration {
        match timer_type {
            TimerType::Run => &self.run,
            TimerType::Compile => &self.compile,
            TimerType::Judge => &self.judge,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RunLangOutput {
    pub tests: JudgeResult,
    pub stderr: String,
    pub timed_out: bool,
    pub runtime: f32,
    pub timers: Timers,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TestPassState {
    /// The test passed
    Pass,
    /// The test failed and caused the entire challenge to fail
    Fail,
    /// This particular test is only informational and has no effect on the pass/fail of the entire challenge
    Info,
    /// This test failed but failure does not necessarily mean the entire challenge failed. Can be used if, for example, you only need 5 out of 6
    /// Tests to pass
    Warning,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DisplayMode {
    #[default]
    Normal,
    Filter,
    Test,
}

#[derive(Serialize, Deserialize, Debug)]
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
        #[serde(rename = "displayMode", default)]
        display_mode: DisplayMode,
        #[serde(rename = "inputSeparator", default = "create_default_sep")]
        input_separator: String,
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
                display_mode: _,
                input_separator: _,
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
