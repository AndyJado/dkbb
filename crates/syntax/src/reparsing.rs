use rowan::GreenNode;
use text_edit::Indel;

use crate::{syntax_error::SyntaxError, syntax_node::SyntaxNode};

pub fn reparse_card(root: &SyntaxNode, edit: &Indel) -> Option<(GreenNode, Vec<SyntaxError>)> {
    use crate::syntax_node::SyntaxKind::*;
    let mut err = vec![];
    let green;
    let current_node = root.covering_element(edit.delete).as_node()?.clone();
    match current_node.kind() {
        GEOMETRY => {
            err.push(SyntaxError::new(
                "don't modify geometry in a text file yet! naughty!".to_string(),
                current_node.text_range(),
            ));
            green = root.green().into_owned();
        }
        _ => {
            let Some(card) = current_node.ancestors().find(|node| node.kind() == CARD) else {return None};
            err.push(SyntaxError::new(
                "got a card!".to_string(),
                card.text_range(),
            ));
            green = root.green().into_owned();
        }
    }
    Some((green, err))
}
