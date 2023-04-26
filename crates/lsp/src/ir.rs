use std::fs;

use syntax::{
    dyna_nodes::SourceFile,
    parse::{parse_text, GreenNode, Parse},
    reparsing::reparse_card,
    syntax_node::SyntaxKind,
};

use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticSeverity, Position, Range, TextDocumentContentChangeEvent,
};

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
        Diagnostic::new(
            range,
            Some(DiagnosticSeverity::WARNING),
            None,
            None,
            msg,
            None,
            None,
        )
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
                    let msg = "The matrix failure material model everybody know at this conference"
                        .to_string();
                    let e = Diagnostic::new(
                        rng,
                        Some(DiagnosticSeverity::INFORMATION),
                        None,
                        None,
                        msg,
                        None,
                        None,
                    );
                    Diagnostics::push(db, e);

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
    let mut line = line - 1;
    let mut col_start: Option<u32> = None;
    let mut col_end: Option<u32> = None;
    let mut word_pos: Vec<(u32, u32)> = vec![];
    let node = card.node(db);
    // let duh = {
    let deck = node.children().into_iter().skip(1).next();
    line += 1;
    let records = deck.unwrap().to_string();
    let strengh = records.lines().skip(5).next().unwrap();
    line += 6;

    let strens: Vec<f32> = strengh
        .split_whitespace()
        .map(|c| c.parse::<f32>().unwrap_or(0.0f32))
        .collect();

    for (i, cr) in strengh.chars().enumerate() {
        match col_start {
            Some(s) => match col_end {
                Some(e) => {
                    word_pos.push((s, e));
                    col_end = None;
                    col_start = None;
                }
                None => {
                    if cr == ' ' {
                        let n = i - 1;
                        col_end = Some(n as u32);
                    }
                }
            },
            None => {
                if cr != ' ' {
                    col_start = Some(i as u32);
                } else {
                    continue;
                };
            }
        }
    }

    for (n, (s, e)) in strens.into_iter().zip(word_pos) {
        if n > 2000.0 {
            let s = Position::new(line, s);
            let e = Position::new(line, e);
            let err = "this strengh is un-neatural".to_string();
            let e = Diagnostic::new(
                Range::new(s, e),
                Some(DiagnosticSeverity::ERROR),
                None,
                None,
                err,
                None,
                None,
            );
            Diagnostics::push(db, e);
        }
    }
}
