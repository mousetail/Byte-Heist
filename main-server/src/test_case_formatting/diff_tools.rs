use std::sync::LazyLock;

use itertools::Itertools;
use similar::{ChangeTag, TextDiff, TextDiffConfig};

use crate::test_case_formatting::filter_iterator_but_keep_context::FilterIteratorButKeepContext;
use crate::test_case_formatting::raw_itemwise_diff::{RawDoubleDiffElement, RawItemwiseDiff};
use crate::test_case_formatting::{Field, FieldKind};

use super::Columns;

pub fn get_diff_elements(left: &str, right: &str, sep: &str, start_column: usize) -> Columns {
    let iterator = FilterIteratorButKeepContext::new(
        RawItemwiseDiff::new(left, right, sep),
        |e| !e.is_boring(),
        RawDoubleDiffElement::new_skipped,
        1,
    );

    let mut height = 0;

    let fields = iterator.flat_map(|item| match item.kind {
        super::raw_itemwise_diff::DoubleDiffKind::Identical(item) => {
            height += 1;
            Box::new(
                [
                    Field {
                        kind: FieldKind::Identical,
                        column: start_column,
                        span: 1,
                        content: item.clone(),
                        row_span: 1,
                    },
                    Field {
                        kind: FieldKind::Identical,
                        column: start_column + 1,
                        span: 1,
                        content: item,
                        row_span: 1,
                    },
                ]
                .into_iter(),
            ) as Box<dyn Iterator<Item = Field>>
        }
        super::raw_itemwise_diff::DoubleDiffKind::Different(left, right) => {
            height += left.len().max(right.len());

            Box::new(left.into_iter().zip_longest(right).flat_map(|pair| {
                let pair_slice = match pair {
                    itertools::EitherOrBoth::Both(a, b) => [Some(a), Some(b)],
                    itertools::EitherOrBoth::Left(a) => [Some(a), None],
                    itertools::EitherOrBoth::Right(b) => [None, Some(b)],
                };

                pair_slice
                    .into_iter()
                    .enumerate()
                    .flat_map(|(index, content)| {
                        content.map(|content| Field {
                            kind: match index {
                                0 => FieldKind::Delete,
                                1 => FieldKind::Insert,
                                _ => unreachable!(),
                            },
                            column: index + start_column,
                            span: 1,
                            content,
                            row_span: 1,
                        })
                    })
            }))
        }
        super::raw_itemwise_diff::DoubleDiffKind::Skipped(number) => {
            height += 1;
            Box::new(
                [Field {
                    kind: FieldKind::Meta,
                    column: start_column,
                    span: 2,
                    content: format!("{} identical lines skipped", number),
                    row_span: 1,
                }]
                .into_iter(),
            )
        }
    });

    Columns {
        fields: fields.collect(),
        column_titles: vec![Some("Output"), Some("Expected")],
        height,
    }
}

pub fn inline_diff(old: &str, new: &str) -> String {
    let old_slices = old.split('\n').map(|k| k.trim_end()).collect::<Vec<_>>();
    let new_slices = new.split('\n').map(|k| k.trim_end()).collect::<Vec<_>>();
    let slices_diff = DIFF_CONFIG.diff_slices(&old_slices, &new_slices);

    let lines_diff = FilterIteratorButKeepContext::new(
        slices_diff.iter_all_changes().map(|c| (c.tag(), c.value())),
        |(tag, _)| matches!(tag, ChangeTag::Delete | ChangeTag::Insert),
        |_| (ChangeTag::Equal, "..."),
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

    diff
}

pub(super) static DIFF_CONFIG: LazyLock<TextDiffConfig> = LazyLock::new(TextDiff::configure);
