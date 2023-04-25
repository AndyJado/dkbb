use syntax::{
    dyna_nodes::SourceFile,
    parse::{parse_text, Parse},
};

use tower_lsp::lsp_types::Diagnostic;

use crate::{helper::range, line_index::LineIndex};

#[salsa::input]
pub struct SourceProgram {
    #[return_ref]
    pub lines: LineIndex,
    pub node: Parse<SourceFile>,
}

#[salsa::accumulator]
pub struct Diagnostics(Diagnostic);

#[salsa::tracked]
pub fn compile(db: &dyn crate::Db, file: SourceProgram) {
    let lines = file.lines(db);
    let cst = file.node(db);
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
