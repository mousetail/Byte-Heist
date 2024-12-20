use std::borrow::Cow;

use common::{RunLangOutput, TestCase, TestPassState};
use serde::Serialize;
use similar::TextDiff;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCaseDisplay {
    columns: Vec<Column>,
    title: Option<Cow<'static, str>>,
    status: TestPassState,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    title: Option<Cow<'static, str>>,
    content: Vec<DiffElement>,
}

#[derive(Serialize)]
struct DiffElement {
    tag: similar::ChangeTag,
    content: String,
}

impl DiffElement {
    fn from_string(e: String) -> Self {
        return DiffElement {
            tag: similar::ChangeTag::Equal,
            content: e,
        };
    }
}

impl TestCaseDisplay {
    pub fn from_test_case(test_case: TestCase) -> Self {
        let diff_generator = TextDiff::configure();

        let columns = match test_case.result_display {
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
                let mut output_diff = vec![];
                let mut expected_diff = vec![];

                for value in diff_generator
                    .diff_slices(
                        output.split(&sep).collect::<Vec<_>>().as_slice(),
                        expected.split(&sep).collect::<Vec<_>>().as_slice(),
                    )
                    .iter_all_changes()
                {
                    let mut text = value.value().to_string();
                    match value.tag() {
                        similar::ChangeTag::Delete => {
                            output_diff.push(DiffElement {
                                tag: similar::ChangeTag::Delete,
                                content: text,
                            });
                            output_diff.push(DiffElement::from_string(sep.clone()));
                        }
                        similar::ChangeTag::Equal => {
                            text.push_str(&sep);
                            expected_diff.push(DiffElement::from_string(text.clone()));
                            output_diff.push(DiffElement::from_string(text));
                        }
                        similar::ChangeTag::Insert => {
                            expected_diff.push(DiffElement {
                                tag: similar::ChangeTag::Insert,
                                content: text,
                            });

                            expected_diff.push(DiffElement::from_string(sep.clone()));
                        }
                    }
                }

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
        };

        TestCaseDisplay {
            columns,
            title: test_case.name.map(Cow::Owned),
            status: test_case.pass,
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
                .collect(),
            passed: value.tests.pass,
            timed_out: value.timed_out,
            judge_error: (!value.stderr.is_empty()).then_some(value.stderr),
        }
    }
}
