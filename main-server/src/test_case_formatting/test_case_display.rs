use std::borrow::Cow;

use common::{ResultDisplay, TestCase, TestPassState};
use serde::Serialize;

use crate::test_case_formatting::{Field, FieldKind, diff_tools::get_diff_elements};

use super::Columns;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCaseDisplay {
    columns: Columns,
    title: Option<Cow<'static, str>>,
    status: TestPassState,
    pub(super) default_visible: bool,
}

impl TestCaseDisplay {
    fn get_columns(result_display: ResultDisplay) -> Columns {
        match result_display {
            common::ResultDisplay::Empty => Columns {
                column_titles: vec![],
                fields: vec![],
                height: 0,
            },
            common::ResultDisplay::Text(e) => Columns {
                column_titles: vec![None],
                fields: vec![Field {
                    column: 0,
                    span: 1,
                    content: e,
                    kind: FieldKind::Identical,
                    row_span: 1,
                }],
                height: 1,
            },
            common::ResultDisplay::Diff {
                output,
                expected,
                input,
                sep,
            } => {
                let mut diff = get_diff_elements(
                    &output,
                    &expected,
                    &sep,
                    match &input {
                        Some(_) => 1,
                        None => 0,
                    },
                );

                if let Some(input) = input {
                    diff.column_titles.insert(0, Some("Input"));
                    diff.fields.insert(
                        0,
                        Field {
                            column: 0,
                            span: 1,
                            content: input,
                            kind: FieldKind::Identical,
                            row_span: diff.height + 1,
                        },
                    );
                }

                diff
            }
            common::ResultDisplay::Run {
                input,
                output,
                error,
            } => match input {
                Some(input) => Columns {
                    column_titles: vec![Some("Input"), Some("Output"), Some("Error")],
                    fields: vec![
                        Field {
                            column: 0,
                            span: 1,
                            content: input,
                            kind: FieldKind::Identical,
                            row_span: 1,
                        },
                        Field {
                            column: 1,
                            span: 1,
                            content: output,
                            kind: FieldKind::Identical,
                            row_span: 1,
                        },
                        Field {
                            column: 2,
                            span: 2,
                            content: error,
                            kind: FieldKind::Identical,
                            row_span: 1,
                        },
                    ],
                    height: 1,
                },
                None => Columns {
                    column_titles: vec![Some("Output"), Some("Error")],
                    fields: vec![
                        Field {
                            column: 1,
                            span: 1,
                            content: output,
                            kind: FieldKind::Identical,
                            row_span: 1,
                        },
                        Field {
                            column: 2,
                            span: 2,
                            content: error,
                            kind: FieldKind::Identical,
                            row_span: 1,
                        },
                    ],
                    height: 1,
                },
            },
        }
    }

    pub fn with_visible(self, visible: bool) -> Self {
        TestCaseDisplay {
            default_visible: visible,
            ..self
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
