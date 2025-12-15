use crate::test_case_formatting::{
    Columns, Field, FieldKind, filter_iterator_but_keep_context::FilterIteratorButKeepContext,
};

pub fn render_filter_display_mode(
    output: String,
    expected: String,
    sep: String,
    input: Option<String>,
    input_separator: String,
) -> Columns {
    let input_parts: Vec<&str> = input
        .as_deref()
        .unwrap_or("")
        .split(&input_separator)
        .collect();

    let output_parts: Vec<(Option<usize>, &str)> = output
        .split(&sep)
        .map(|i| (input_parts.iter().position(|&k| k == i), i))
        .collect();
    let expected_parts: Vec<(Option<usize>, &str)> = expected
        .split(&sep)
        .map(|i| (input_parts.iter().position(|&k| k == i), i))
        .collect();

    let mut fields: Vec<Vec<Field>> = vec![];

    let mut output_index = 0;
    let mut expected_index = 0;

    while output_index < output_parts.len() || expected_index < expected_parts.len() {
        let output_value = output_parts
            .get(output_index)
            .copied()
            .unwrap_or((Some(input_parts.len() - 1), ""));
        let expected_value = expected_parts
            .get(expected_index)
            .copied()
            .unwrap_or((Some(input_parts.len() - 1), ""));

        match (output_value, expected_value) {
            ((Some(left), _), (Some(right), _)) if left == right => {
                output_index += 1;
                expected_index += 1;

                fields.push(vec![
                    Field {
                        column: 0,
                        span: 1,
                        row_span: 1,
                        content: input_parts[left].to_owned(),
                        kind: FieldKind::Identical,
                    },
                    Field {
                        column: 1,
                        span: 1,
                        row_span: 1,
                        content: input_parts[left].to_owned(),
                        kind: FieldKind::Identical,
                    },
                    Field {
                        column: 2,
                        span: 1,
                        row_span: 1,
                        content: input_parts[left].to_owned(),
                        kind: FieldKind::Identical,
                    },
                ])
            }
            ((None, left_str), (None, right_str)) if left_str == right_str => {
                output_index += 1;
                expected_index += 1;
                fields.push(vec![
                    Field {
                        column: 1,
                        span: 1,
                        row_span: 1,
                        content: left_str.to_owned(),
                        kind: FieldKind::Identical,
                    },
                    Field {
                        column: 2,
                        span: 1,
                        row_span: 1,
                        content: right_str.to_owned(),
                        kind: FieldKind::Identical,
                    },
                ])
            }
            ((None, left_str), (None, right_str)) => {
                output_index += 1;
                expected_index += 1;
                fields.push(vec![
                    Field {
                        column: 1,
                        span: 1,
                        row_span: 1,
                        content: left_str.to_owned(),
                        kind: FieldKind::Delete,
                    },
                    Field {
                        column: 2,
                        span: 1,
                        row_span: 1,
                        content: right_str.to_owned(),
                        kind: FieldKind::Insert,
                    },
                ])
            }

            ((None, left_str), _) => {
                output_index += 1;
                fields.push(vec![Field {
                    column: 1,
                    span: 1,
                    row_span: 1,
                    content: left_str.to_owned(),
                    kind: FieldKind::Delete,
                }])
            }
            (_, (None, right_str)) => {
                expected_index += 1;
                fields.push(vec![Field {
                    column: 2,
                    span: 1,
                    row_span: 1,
                    content: right_str.to_owned(),
                    kind: FieldKind::Insert,
                }])
            }
            ((Some(left), left_str), (Some(right), _)) if left <= right => {
                output_index += 1;

                fields.push(vec![
                    Field {
                        column: 0,
                        span: 1,
                        row_span: 1,
                        content: input_parts[left].to_owned(),
                        kind: FieldKind::Identical,
                    },
                    Field {
                        column: 1,
                        span: 1,
                        row_span: 1,
                        content: left_str.to_owned(),
                        kind: FieldKind::Delete,
                    },
                ])
            }
            (_, (Some(right), right_str)) => {
                expected_index += 1;

                fields.push(vec![
                    Field {
                        column: 0,
                        span: 1,
                        row_span: 1,
                        content: input_parts[right].to_owned(),
                        kind: FieldKind::Identical,
                    },
                    Field {
                        column: 2,
                        span: 1,
                        row_span: 1,
                        content: right_str.to_owned(),
                        kind: FieldKind::Insert,
                    },
                ])
            }
        }
    }

    let filtered_fields: Vec<Field> = FilterIteratorButKeepContext::new(
        fields.into_iter(),
        |i| i.iter().any(|k| k.kind != FieldKind::Identical),
        |rows_skipped| {
            vec![Field {
                column: 0,
                span: 3,
                row_span: 1,
                content: format!("{rows_skipped} identical lines skipped"),
                kind: FieldKind::Meta,
            }]
        },
        1,
    )
    .flatten()
    .collect();

    Columns {
        column_titles: vec![Some("Input"), Some("Output"), Some("Expected")],
        height: filtered_fields.len(),
        fields: filtered_fields,
    }
}
