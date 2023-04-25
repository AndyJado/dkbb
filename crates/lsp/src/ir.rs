use std::fs;

use syntax::{
    dyna_nodes::SourceFile,
    parse::{parse_text, Parse},
};

use tower_lsp::lsp_types::{Diagnostic, Range};

use crate::{helper::range, line_index::LineIndex};

#[salsa::input]
pub struct Vfs {
    pub url: String,
}

#[salsa::tracked]
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
pub fn read(db: &dyn crate::Db, id: Vfs) -> SourceProgram {
    let file = fs::read_to_string(id.url(db));
    let f = match file {
        Ok(f) => f,
        Err(_) => {
            let err =
                Diagnostic::new_simple(Range::default(), "can't read file from uri".to_string());
            Diagnostics::push(db, err);
            String::new()
        }
    };
    let lines = LineIndex::new(&f);
    SourceProgram::new(db, f, lines)
}

#[salsa::tracked]
pub fn parse(db: &dyn crate::Db, source: SourceProgram) -> Program {
    let f = source.file(db);
    Program::new(db, parse_text(&f))
}

#[salsa::tracked]
pub fn compile(db: &dyn crate::Db, input: Vfs) {
    // Get the source text from the database
    let file = read(db, input);
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
