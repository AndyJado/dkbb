use rowan::{GreenNode, TextRange};
use text_edit::Indel;

use crate::{syntax_error::SyntaxError, syntax_node::SyntaxNode};

pub fn reparse_card(root: &SyntaxNode, edit: &Indel) -> Option<(GreenNode, Vec<SyntaxError>)> {
    use crate::syntax_node::SyntaxKind::{self, *};
    let mut err = vec![];
    let green;
    let current_node = root.covering_element(edit.delete).as_node()?.clone();
    let mom_is = |c: SyntaxKind| current_node.ancestors().find(|node| node.kind() == c);

    if let Some(node) = mom_is(GEOMETRY) {
        err.push(SyntaxError::new(
            "don't edit geometry yet, naughty!".to_string(),
            TextRange::default(),
        ))
    }

    if let Some(node) = mom_is(CARD) {
        err.push(SyntaxError::new(
            "got a card!".to_string(),
            TextRange::empty(node.text_range().start()),
        ));
    } else {
        for i in current_node.ancestors() {
            let duh = i.kind();
            err.push(SyntaxError::new(
                format!("{:?}", duh),
                TextRange::empty(current_node.text_range().start()),
            ));
        }
    }
    // green = root.green().into_owned();
    green = root.green().into_owned();
    Some((green, err))
}
