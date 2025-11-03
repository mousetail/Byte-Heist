use std::collections::VecDeque;

use similar::{DiffOp, capture_diff};

pub(super) enum DoubleDiffKind {
    Identical(String),
    Different(Vec<String>, Vec<String>),
    Skipped(usize),
}
pub(super) struct RawDoubleDiffElement {
    #[allow(unused)]
    left_line_number: usize,

    #[allow(unused)]
    right_line_number: usize,
    pub(super) kind: DoubleDiffKind,
}

impl RawDoubleDiffElement {
    pub fn is_boring(&self) -> bool {
        matches!(self.kind, DoubleDiffKind::Identical(_))
    }

    pub fn new_skipped(amount: usize) -> Self {
        RawDoubleDiffElement {
            left_line_number: 0,
            right_line_number: 0,
            kind: DoubleDiffKind::Skipped(amount),
        }
    }
}

pub(super) struct RawItemwiseDiff {
    left_split: Vec<String>,
    right_split: Vec<String>,
    diff: VecDeque<DiffOp>,
    index: usize,
}

impl RawItemwiseDiff {
    pub(super) fn new(left: &str, right: &str, sep: &str) -> RawItemwiseDiff {
        let strip_string = |k: &str| format!("{}{sep}", k.trim_end());

        let left_split = left
            .trim_end()
            .split(&sep)
            .map(strip_string)
            .collect::<Vec<_>>();
        let right_split = right
            .trim_end()
            .split(&sep)
            .map(strip_string)
            .collect::<Vec<_>>();

        let diff = capture_diff(
            similar::Algorithm::Myers,
            &left_split,
            0..left_split.len(),
            &right_split,
            0..right_split.len(),
        )
        .into();

        RawItemwiseDiff {
            left_split,
            right_split,
            diff,
            index: 0,
        }
    }
}

impl Iterator for RawItemwiseDiff {
    type Item = RawDoubleDiffElement;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        match self.diff.get(self.index - 1) {
            Some(diff_op) => match *diff_op {
                DiffOp::Equal {
                    old_index,
                    new_index,
                    len,
                } => {
                    if len > 1 {
                        self.index -= 1;

                        self.diff[self.index] = DiffOp::Equal {
                            old_index: old_index + 1,
                            new_index: new_index + 1,
                            len: len - 1,
                        }
                    }

                    Some(RawDoubleDiffElement {
                        kind: DoubleDiffKind::Identical(self.left_split[old_index].to_string()),
                        left_line_number: old_index,
                        right_line_number: new_index,
                    })
                }
                DiffOp::Delete {
                    old_index,
                    old_len,
                    new_index,
                } => Some(RawDoubleDiffElement {
                    kind: DoubleDiffKind::Different(
                        self.left_split[old_index..old_index + old_len].to_vec(),
                        vec![],
                    ),
                    left_line_number: old_index,
                    right_line_number: new_index,
                }),
                DiffOp::Insert {
                    old_index,
                    new_index,
                    new_len,
                } => Some(RawDoubleDiffElement {
                    kind: DoubleDiffKind::Different(
                        vec![],
                        self.right_split[new_index..new_index + new_len].to_vec(),
                    ),
                    left_line_number: old_index,
                    right_line_number: new_index,
                }),
                DiffOp::Replace {
                    old_index,
                    old_len,
                    new_index,
                    new_len,
                } => Some(RawDoubleDiffElement {
                    kind: DoubleDiffKind::Different(
                        self.left_split[old_index..old_index + old_len].to_vec(),
                        self.right_split[new_index..new_index + new_len].to_vec(),
                    ),
                    left_line_number: old_index,
                    right_line_number: new_index,
                }),
            },
            None => None,
        }
    }
}
