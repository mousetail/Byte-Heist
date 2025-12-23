mod display_modes;
mod test_case_display;

use common::{RunLangOutput, Timers};
use serde::Serialize;
use test_case_display::TestCaseDisplay;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputDisplay {
    tests: Vec<TestCaseDisplay>,
    passed: bool,
    timed_out: bool,
    judge_error: Option<String>,
    timers: Timers,
}

impl From<RunLangOutput> for OutputDisplay {
    fn from(value: RunLangOutput) -> Self {
        OutputDisplay {
            tests: value
                .tests
                .test_cases
                .into_iter()
                .map(TestCaseDisplay::from_test_case)
                .map(|e| {
                    // If the test passes, hide all info boxes
                    if value.tests.pass {
                        e.with_visible(false)
                    } else {
                        e
                    }
                })
                .collect(),
            passed: value.tests.pass,
            timed_out: value.timed_out,
            judge_error: (!value.stderr.is_empty()).then_some(value.stderr),
            timers: value.timers,
        }
    }
}
