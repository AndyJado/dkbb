use std::{fs};

use syntax::{
    dyna_nodes::SourceFile,
    parse::{parse_text, GreenNode, Parse},
    reparsing::reparse_card,
    syntax_node::{SyntaxKind},
};


use tower_lsp::lsp_types::{Diagnostic, Position, Range, TextDocumentContentChangeEvent};

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

#[salsa::tracked]
pub struct Card {
    pub node: GreenNode,
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
        let Some((_green,err)) = reparse_card(&cst.syntax_node(), &i) else {continue;};
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
        let rng = range(&lines, i.text_range());
        match i.kind() {
            SyntaxKind::GEOMETRY => {
                Diagnostics::push(db, Diagnostic::new_simple(rng, "here a geo!".to_string()))
            }
            SyntaxKind::CARD => {
                let mut card_node = i.descendants();
                let Some(kwd) = card_node.find(|kd| kd.kind() == SyntaxKind::KEYWORD) else {continue};
                let (wd, rg) = (kwd.text().to_string(), kwd.text_range());
                if wd.trim() == "*MAT_ENHANCED_COMPOSITE_DAMAGE_TITLE" {
                    let rng = range(lines, rg);
                    Diagnostics::push(
                        db,
                        Diagnostic::new_simple(rng, "sorry I only recogonize this one".to_string()),
                    );
                    match card_node.next() {
                        Some(nd) => {
                            if nd.kind() == SyntaxKind::DECK {
                                let rng: Range = range(lines, nd.text_range());
                                let green = nd.green().into_owned();
                                mat_54(db, Card::new(db, green), rng.start.line)
                            }
                        }
                        None => {}
                    }
                }
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

#[salsa::tracked]
pub fn mat_54(db: &dyn crate::Db, card: Card, line: u32) {
    let line = line - 1;
    let node = card.node(db);
    let pos = Position {
        line,
        ..Default::default()
    };
    let e = Diagnostic::new_simple(Range::new(pos, pos), format!("{:?}", node.kind()));
    Diagnostics::push(db, e);
}
