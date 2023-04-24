use syntax::{dyna_nodes::SourceFile, parse::Parse, syntax_error::SyntaxError};

#[salsa::input]
pub struct Program {
    pub node: Parse<SourceFile>,
}

#[salsa::accumulator]
pub struct Diagnostics(SyntaxError);

#[salsa::tracked]
pub fn compile(db: &dyn crate::Db, source: Program) {
    // Get the source text from the database
    let cst = source.node(db);
    let err = cst.errors;
    for e in err.iter() {
        Diagnostics::push(db, e.clone())
    }
}
