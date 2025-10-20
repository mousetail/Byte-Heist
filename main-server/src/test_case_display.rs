use std::{borrow::Cow, sync::LazyLock};

use common::{ResultDisplay, RunLangOutput, TestCase, TestPassState};
use serde::Serialize;
use similar::{ChangeTag, TextDiff, TextDiffConfig};

use crate::filter_iterator_but_keep_context::FilterIteratorButKeepContext;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCaseDisplay {
    columns: Vec<Column>,
    title: Option<Cow<'static, str>>,
    status: TestPassState,
    default_visible: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    title: Option<Cow<'static, str>>,
    content: Vec<DiffElement>,
}

#[derive(Serialize, PartialEq, Eq)]
pub(crate) struct DiffElement {
    tag: similar::ChangeTag,
    content: String,
}

impl DiffElement {
    fn from_string(e: String) -> Self {
        DiffElement {
            tag: similar::ChangeTag::Equal,
            content: e,
        }
    }
}

pub fn get_diff_elements(
    left: String,
    right: String,
    sep: &str,
) -> (Vec<DiffElement>, Vec<DiffElement>) {
    let mut output_diff = vec![];
    let mut expected_diff = vec![];

    let left_split = left.split(&sep).map(|k| k.trim_end()).collect::<Vec<_>>();
    let right_map = right.split(&sep).map(|k| k.trim_end()).collect::<Vec<_>>();

    let slices_diff = DIFF_CONFIG.diff_slices(&left_split, &right_map);

    for (tag, value) in FilterIteratorButKeepContext::new(
        slices_diff.iter_all_changes().map(|i| (i.tag(), i.value())),
        |(tag, _)| tag != ChangeTag::Equal,
        (ChangeTag::Equal, "..."),
        3,
    ) {
        let mut text = value.to_string();
        match tag {
            similar::ChangeTag::Delete => {
                output_diff.push(DiffElement {
                    tag: similar::ChangeTag::Delete,
                    content: text,
                });
                output_diff.push(DiffElement::from_string(sep.to_owned()));
            }
            similar::ChangeTag::Equal => {
                text.push_str(sep);
                expected_diff.push(DiffElement::from_string(text.clone()));
                output_diff.push(DiffElement::from_string(text));
            }
            similar::ChangeTag::Insert => {
                expected_diff.push(DiffElement {
                    tag: similar::ChangeTag::Insert,
                    content: text,
                });

                expected_diff.push(DiffElement::from_string(sep.to_owned()));
            }
        }
    }

    (output_diff, expected_diff)
}

impl TestCaseDisplay {
    fn get_columns(result_display: ResultDisplay) -> Vec<Column> {
        match result_display {
            common::ResultDisplay::Empty => vec![],
            common::ResultDisplay::Text(e) => vec![Column {
                title: None,
                content: vec![DiffElement::from_string(e)],
            }],
            common::ResultDisplay::Diff {
                output,
                expected,
                input,
                sep,
            } => {
                let (output_diff, expected_diff) = get_diff_elements(output, expected, &sep);
                input
                    .map(|e| Column {
                        title: Some(Cow::Borrowed("Input")),
                        content: vec![DiffElement::from_string(e)],
                    })
                    .into_iter()
                    .chain(vec![
                        Column {
                            title: Some(Cow::Borrowed("Output")),
                            content: output_diff,
                        },
                        Column {
                            title: Some(Cow::Borrowed("Expected")),
                            content: expected_diff,
                        },
                    ])
                    .collect()
            }
            common::ResultDisplay::Run {
                input,
                output,
                error,
            } => vec![
                Column {
                    title: Some(Cow::Borrowed("Input")),
                    content: vec![DiffElement::from_string(input.unwrap_or_default())],
                },
                Column {
                    title: Some(Cow::Borrowed("Output")),
                    content: vec![DiffElement::from_string(output)],
                },
                Column {
                    title: Some(Cow::Borrowed("Error")),
                    content: vec![DiffElement::from_string(error)],
                },
            ],
        }
    }

    fn get_default_visible(test_case: &TestCase) -> bool {
        match test_case.pass {
            TestPassState::Pass => false,
            TestPassState::Fail => true,
            TestPassState::Info => match &test_case.result_display {
                ResultDisplay::Run {
                    input: _,
                    output: _,
                    error,
                } => !error.is_empty(),
                _ => true,
            },
            TestPassState::Warning => true,
        }
    }

    pub fn from_test_case(test_case: TestCase) -> Self {
        let default_visible = Self::get_default_visible(&test_case);
        let columns = Self::get_columns(test_case.result_display);

        TestCaseDisplay {
            columns,
            title: test_case.name.map(Cow::Owned),
            status: test_case.pass,
            default_visible,
        }
    }
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
                        TestCaseDisplay {
                            default_visible: false,
                            ..e
                        }
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

pub fn inline_diff(old: &str, new: &str) -> String {
    let old_slices = old.split('\n').map(|k| k.trim_end()).collect::<Vec<_>>();
    let new_slices = new.split('\n').map(|k| k.trim_end()).collect::<Vec<_>>();
    let slices_diff = DIFF_CONFIG.diff_slices(&old_slices, &new_slices);

    let lines_diff = FilterIteratorButKeepContext::new(
        slices_diff.iter_all_changes().map(|c| (c.tag(), c.value())),
        |(tag, _)| matches!(tag, ChangeTag::Delete | ChangeTag::Insert),
        (ChangeTag::Equal, "..."),
        3,
    );

    let mut diff = ["```diff\n"]
        .into_iter()
        .chain(lines_diff.flat_map(|(tag, value)| {
            [
                match tag {
                    ChangeTag::Delete => "- ",
                    ChangeTag::Insert => "+ ",
                    ChangeTag::Equal => "  ",
                },
                value,
                "\n",
            ]
        }))
        .collect::<String>();
    diff.truncate(1500);
    diff.push_str("```");

    return diff;
}

static DIFF_CONFIG: LazyLock<TextDiffConfig> = LazyLock::new(TextDiff::configure);
