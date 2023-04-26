use rowan::{GreenNode, TextRange};
use text_edit::Indel;

use crate::{syntax_error::SyntaxError, syntax_node::SyntaxNode};

pub fn reparse_card(root: &SyntaxNode, edit: &Indel) -> Option<(GreenNode, Vec<SyntaxError>)> {
    use crate::syntax_node::SyntaxKind::{self, *};
    let mut err = vec![];
    let range = edit.delete;
    let current = root.covering_element(range);
    let current_node = current.as_node();
    let (message, range) = match current_node {
        Some(node) => (node.text().to_string(), node.text_range()),
        None => {
            let tk = current.into_token().unwrap();
            (tk.text().to_string(), range)
        }
    };
    err.push(SyntaxError::new(message, range));
    Some((root.green().into_owned(), err))
}
