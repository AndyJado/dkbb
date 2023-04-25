use std::fs;

use syntax::{
    dyna_nodes::SourceFile,
    parse::{parse_text, Parse},
};

use tower_lsp::lsp_types::{Diagnostic, Range};

use crate::{helper::range, line_index::LineIndex};

#[salsa::input]
pub struct SourceProgram {
    #[return_ref]
    pub file: String,
    pub lines: LineIndex,
}

#[salsa::tracked]
pub struct Program {
    pub node: Parse<SourceFile>,
}

#[salsa::accumulator]
pub struct Diagnostics(Diagnostic);

#[salsa::tracked]
pub fn parse(db: &dyn crate::Db, source: SourceProgram) -> Program {
    let f = source.file(db);
    Program::new(db, parse_text(&f))
}

#[salsa::tracked]
pub fn compile(db: &dyn crate::Db, file: SourceProgram) {
    // Get the source text from the database
    let lines = file.lines(db);
    let program = parse(db, file);
    let cst = program.node(db);
    let err = cst.errors;
    let diags = err.iter().map(|c| {
        let range = range(&lines, c.range());
        let msg = c.to_string();
        Diagnostic::new_simple(range, msg)
    });
    let _node = cst.green;
    for e in diags {
        Diagnostics::push(db, e)
    }
}
