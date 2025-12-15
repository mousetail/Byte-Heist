use itertools::Itertools;

use crate::test_case_formatting::{
    Columns, Field, FieldKind, filter_iterator_but_keep_context::FilterIteratorButKeepContext,
};

pub fn render_test_case_display_mode(
    output: String,
    expected: String,
    sep: String,
    input: Option<String>,
    input_separator: String,
) -> Columns {
    let input_separator = input_separator.as_str();

    let iter = input
        .as_ref()
        .map(|i| i.as_str())
        .unwrap_or("")
        .split(input_separator)
        .zip_longest(output.split(&sep))
        .zip_longest(expected.split(&sep))
        .map(|b| {
            let (left_and_center, right) = b.left_and_right();
            let (left, center) = left_and_center
                .map(|i| i.left_and_right())
                .unwrap_or((None, None));

            return (
                left.unwrap_or(""),
                center.unwrap_or(""),
                right.unwrap_or(""),
            );
        });

    let fields: Vec<Field> = FilterIteratorButKeepContext::new(
        iter.map(|(input, output, expected)| {
            vec![
                Field {
                    column: 0,
                    span: 1,
                    row_span: 1,
                    content: input.to_owned(),
                    kind: crate::test_case_formatting::FieldKind::Identical,
                },
                Field {
                    column: 1,
                    span: 1,
                    row_span: 1,
                    content: output.to_owned(),
                    kind: if output == expected {
                        FieldKind::Identical
                    } else {
                        FieldKind::Delete
                    },
                },
                Field {
                    column: 2,
                    span: 1,
                    row_span: 1,
                    content: output.to_owned(),
                    kind: if output == expected {
                        FieldKind::Identical
                    } else {
                        FieldKind::Insert
                    },
                },
            ]
        }),
        |i| i[1].kind != FieldKind::Identical,
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
        height: fields.len(),
        fields,
    }
}
