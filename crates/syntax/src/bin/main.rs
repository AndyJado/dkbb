use syntax::{
    ast::AstNode,
    dyna_nodes::KeyWord,
    parse::parse_text,
    syntax_node::{print, SyntaxNode},
};

fn main() {
    let source_code = "*MAT_ENHANCED_COMPOSITE_DAMAGE_TITLE
Default MAT54-MAT55 MAT_ENHANCED_COMPOSITE_DAMAGE
       132  1.577E-9    78660.    11385.    11385.       0.3        0.        0.
      52.9      52.9      52.9        0.       2.0
                                    0.94     -0.06        0.
                                      0.        0.        1.        0.        0.
        0.        0.        2.      0.05        1.        0.        0.        0.
  7000000.  7000000.  2000000.  2000000.  7000000.      55.0      0.05
        0.        0.        0.        0.        1.
        0.        0.        0.        0.        0.         0        1.
";
    let parse = parse_text(source_code);
    dbg!(&parse.errors);
    // assert!(parse.errors().is_empty());
    let file = parse.tree();
    for i in file.syntax().descendants() {
        println!("{:#?},{:#?}", i.kind(), i.text_range());
        // let Some(mut kwd) = KeyWord::cast(i) else {continue};
        // kwd.play();
    }
}

fn print_cst_from_env() {
    use syntax::helpers::*;
    let file = env_read();
    let mut now = Now::new();
    let res = parse_text(&file);
    now.now("parsing");
    let node = SyntaxNode::new_root(res.green.clone());
    syntax::syntax_node::print(0, node.into());
    now.now("printing");
}
