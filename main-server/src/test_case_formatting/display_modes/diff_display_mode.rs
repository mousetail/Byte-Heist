use crate::test_case_formatting::{Columns, Field, FieldKind, get_diff_elements};

pub fn render_diff_display_mode(
    output: String,
    expected: String,
    sep: String,
    input: Option<String>,
) -> Columns {
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
