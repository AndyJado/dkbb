use std::sync::Arc;

use syntax::{
    dyna_nodes::SourceFile,
    parse::{parse_text, Parse},
};

#[salsa::tracked]
pub struct Program {
    node: Parse<SourceFile>,
}

#[salsa::input]
pub struct SourceProgram {
    #[return_ref]
    pub text: String,
}

#[salsa::accumulator]
pub struct Diagnostics(Diagnostic);

#[derive(Clone, Debug)]
pub struct Diagnostic {
    // pub start: usize,
    // pub end: usize,
    pub message: String,
}

#[salsa::tracked]
pub fn compile(db: &dyn crate::Db, source_program: SourceProgram) {
    parse(db, source_program);
}

#[salsa::tracked]
pub fn parse(db: &dyn crate::Db, source: SourceProgram) -> Program {
    // Get the source text from the database
    let source_text = source.text(db);
    let cst = parse_text(&source_text);
    let message = match cst.errors.last() {
        Some(e) => e.to_string(),
        None => "no err".to_string(),
    };
    let diag = Diagnostic { message };
    Diagnostics::push(db, diag);
    Program::new(db, cst)
}
