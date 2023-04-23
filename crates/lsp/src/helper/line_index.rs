use anyhow::format_err;
use anyhow::Result;
use std::iter;

// FIXME: removed pest: use pest::{error::LineColLocation, Span};
use syntax::parse::TextRange;
use syntax::parse::TextSize;
use tower_lsp::lsp_types::{Position, Range};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineIndex {
    /// Offset the beginning of each line, zero-based.
    pub newlines: Vec<TextSize>,
}

/// Line/Column information in native, utf8 format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LineCol {
    /// Zero-based
    pub line: u32,
    /// Zero-based utf8 offset
    pub col: u32,
}

impl LineIndex {
    pub fn new(text: &str) -> LineIndex {
        let mut newlines = Vec::with_capacity(16);
        newlines.push(TextSize::from(0));

        let mut curr_row = 0.into();
        let mut curr_col: TextSize = 0.into();
        let mut line = 0;
        for c in text.chars() {
            let c_len = TextSize::of(c);
            curr_row += c_len;
            if c == '\n' {
                newlines.push(curr_row);

                // Prepare for processing the next line
                curr_col = 0.into();
                line += 1;
                continue;
            }

            curr_col += c_len;
        }

        // Save any utf-16 characters seen in the last line
        LineIndex { newlines }
    }

    pub fn line_col(&self, offset: TextSize) -> LineCol {
        let line = self.newlines.partition_point(|&it| it <= offset) - 1;
        let line_start_offset = self.newlines[line];
        let col = offset - line_start_offset;
        LineCol {
            line: line as u32,
            col: col.into(),
        }
    }

    pub fn offset(&self, line_col: LineCol) -> Option<TextSize> {
        self.newlines
            .get(line_col.line as usize)
            .map(|offset| offset + TextSize::from(line_col.col))
    }

    pub fn lines(&self, range: TextRange) -> impl Iterator<Item = TextRange> + '_ {
        let lo = self.newlines.partition_point(|&it| it < range.start());
        let hi = self.newlines.partition_point(|&it| it <= range.end());
        let all = iter::once(range.start())
            .chain(self.newlines[lo..hi].iter().copied())
            .chain(iter::once(range.end()));

        all.clone()
            .zip(all.skip(1))
            .map(|(lo, hi)| TextRange::new(lo, hi))
            .filter(|it| !it.is_empty())
    }
}

pub fn position(line_index: &LineIndex, offset: TextSize) -> Position {
    let line_col = line_index.line_col(offset);
    Position::new(line_col.line, line_col.col)
}

pub fn range(line_index: &LineIndex, range: TextRange) -> Range {
    let start = position(line_index, range.start());
    let end = position(line_index, range.end());
    Range::new(start, end)
}

pub fn offset(line_index: &LineIndex, position: Position) -> Result<TextSize> {
    let line_col = LineCol {
        line: position.line,
        col: position.character,
    };
    let text_size = line_index
        .offset(line_col)
        .ok_or_else(|| format_err!("Invalid offset"))?;
    Ok(text_size)
}

pub fn text_range(line_index: &LineIndex, range: Range) -> Result<TextRange> {
    let start = offset(line_index, range.start)?;
    let end = offset(line_index, range.end)?;
    match end < start {
        true => Err(format_err!("Invalid Range").into()),
        false => Ok(TextRange::new(start, end)),
    }
}
