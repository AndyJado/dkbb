use crate::line_index::LineCol;
use anyhow::format_err;
use anyhow::Result;
use line_index::LineIndex;
use syntax::parse::TextRange;
use syntax::parse::TextSize;
use text_edit::Indel;
use text_edit::TextEdit;

use tower_lsp::lsp_types::TextDocumentContentChangeEvent;
use tower_lsp::lsp_types::{Position, Range};

use crate::line_index;

pub type LspEdit = tower_lsp::lsp_types::TextEdit;

// XXX:untested
pub fn user_edit(line_index: &LineIndex, changes: Vec<TextDocumentContentChangeEvent>) -> TextEdit {
    let mut edits = TextEdit::builder();
    for c in changes {
        let (Some(range), new_text) = (c.range, c.text) else {continue};
        let edit = to_indel(line_index, LspEdit { range, new_text });
        edits.indel(edit);
    }
    edits.finish()
}

pub fn to_indel(line_index: &LineIndex, edit: LspEdit) -> Indel {
    let LspEdit { range, new_text } = edit;
    Indel {
        insert: new_text,
        delete: text_range(line_index, range).unwrap(),
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
