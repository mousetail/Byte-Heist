mod diff_tools;
mod filter_iterator_but_keep_context;
mod raw_itemwise_diff;
mod test_case_display;

use common::RunLangOutput;
pub use diff_tools::{get_diff_elements, inline_diff};
use serde::Serialize;
use test_case_display::TestCaseDisplay;

#[derive(Serialize, PartialEq, Eq, Clone)]
pub struct Columns {
    column_titles: Vec<Option<&'static str>>,
    fields: Vec<Field>,
    height: usize,
}

#[derive(Serialize, PartialEq, Eq, Clone)]
struct Field {
    column: usize,
    span: usize,
    row_span: usize,
    content: String,
    kind: FieldKind,
}

#[derive(Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum FieldKind {
    Insert,
    Delete,
    Identical,
    Meta,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputDisplay {
    tests: Vec<TestCaseDisplay>,
    passed: bool,
    timed_out: bool,
    judge_error: Option<String>,
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
        }
    }
}
