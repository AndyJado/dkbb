use rowan::{GreenNode, TextRange};
use text_edit::Indel;

use crate::{syntax_error::SyntaxError, syntax_node::SyntaxNode};

pub fn reparse_card(root: &SyntaxNode, edit: &Indel) -> Option<(GreenNode, Vec<SyntaxError>)> {
    use crate::syntax_node::SyntaxKind::{self, *};
    let mut err = vec![];
    let green;
    let range = edit.delete;
    let current_node = root.covering_element(range).as_node().cloned();

    let (message, range) = match current_node {
        Some(node) => (node.text().to_string(), node.text_range()),
        None => ("edit too much at a time".to_string(), range),
    };
    err.push(SyntaxError::new(message, range));
    green = root.green().into_owned();
    Some((green, err))
}
