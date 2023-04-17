use crate::{
    ast::AstNode,
    syntax_node::{SyntaxKind, SyntaxNode},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceFile {
    pub(crate) syntax: SyntaxNode,
}

// should be generated code !
impl AstNode for SourceFile {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::ROOT
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyWord {
    pub(crate) syntax: SyntaxNode,
}

impl KeyWord {
    pub fn play(&mut self) {
        let words = self
            .syntax
            .children_with_tokens()
            .filter(|c| c.kind() == SyntaxKind::WORD);
        for i in words {
            println!("{:?}", i.as_token().expect("as_token").text())
        }
    }
}

impl AstNode for KeyWord {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::KEYWORD
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
