use anyhow::format_err;
use anyhow::Result;
use ide_db::line_index::LineCol;
use ide_db::line_index::LineIndex;
use syntax::parse::TextRange;
use syntax::parse::TextSize;
use tower_lsp::lsp_types::{Position, Range};

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
