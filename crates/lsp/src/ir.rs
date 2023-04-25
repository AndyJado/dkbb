use std::fs;

use syntax::{
    dyna_nodes::SourceFile,
    parse::{parse_text, Parse},
    reparsing::reparse_card,
    syntax_node::SyntaxKind,
};

use text_edit::TextEdit;
use tower_lsp::lsp_types::{Diagnostic, TextDocumentContentChangeEvent};

use crate::{
    helper::{range, user_edit},
    line_index::LineIndex,
};

#[salsa::input]
pub struct Source {
    #[return_ref]
    path: String,
}

#[salsa::tracked]
pub struct SourceProgram {
    #[return_ref]
    pub lines: LineIndex,
    pub node: Parse<SourceFile>,
}

#[salsa::input]
pub struct Diff {
    edits: Vec<TextDocumentContentChangeEvent>,
}

#[salsa::accumulator]
pub struct Diagnostics(Diagnostic);

#[salsa::tracked]
pub fn parse(db: &dyn crate::Db, path: Source) -> SourceProgram {
    let file = fs::read_to_string(path.path(db));
    let f = match file {
        Ok(f) => f,
        Err(_) => String::new(),
    };
    let lines = LineIndex::new(&f);
    let node = parse_text(&f);
    SourceProgram::new(db, lines, node)
}

#[salsa::tracked]
pub fn foo(db: &dyn crate::Db, source: SourceProgram, diff: Diff) {
    let (cst, lines) = (source.node(db), source.lines(db));
    let edits = user_edit(lines, diff.edits(db));
    for i in edits {
        let Some((green,err)) = reparse_card(&cst.syntax_node(), &i) else {continue;};
        let diags = err.iter().map(|c| {
            let range = range(&lines, c.range());
            let msg = c.to_string();
            Diagnostic::new_simple(range, msg)
        });
        for e in diags {
            Diagnostics::push(db, e)
        }
    }
}

#[salsa::tracked]
pub fn compile(db: &dyn crate::Db, source: Source, edit: Option<Diff>) {
    let source = parse(db, source);
    let (cst, lines) = (source.node(db), source.lines(db));
    let err = cst.errors.clone();
    let diags = err.iter().map(|c| {
        let range = range(&lines, c.range());
        let msg = c.to_string();
        Diagnostic::new_simple(range, msg)
    });

    for i in cst.to_syntax().syntax_node().descendants() {
        let range = range(&lines, i.text_range());
        match i.kind() {
            SyntaxKind::GEOMETRY => {
                Diagnostics::push(db, Diagnostic::new_simple(range, "here a geo!".to_string()))
            }
            SyntaxKind::CARD => {
                let kwd = i.descendants().find(|kd| kd.kind() == SyntaxKind::KEYWORD);
                Diagnostics::push(db, Diagnostic::new_simple(range, "here a kwd!".to_string()))
            }
            _ => {}
        }
    }

    for e in diags {
        Diagnostics::push(db, e)
    }

    if let Some(diff) = edit {
        foo(db, source, diff)
    };
}
