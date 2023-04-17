use logos::Logos;
use rowan::TextSize;
use rowan::{GreenNode, GreenNodeBuilder, Language, NodeOrToken};

use crate::syntax_error::SyntaxError;

#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u16)]
#[allow(dead_code)]
pub enum SyntaxKind {
    #[regex(r" +")]
    WHITESPACE = 0,
    #[regex(r"[[:alpha:]][[[:alnum:]]-\(\)\.]*")]
    WORD,
    #[regex(r"(\r\n)|(\n)")]
    NEWLINE,
    #[token("*")]
    ASTERISK,
    #[regex(r"\$.*\n")]
    COMMENT,
    #[token("_")]
    UNDERSCODE,
    #[regex(r"-?\d+(?:\.\d*)?(?:[eE][+-]?\d+)?")]
    NUMBER,

    #[regex(r"\*NODE[^\*]*")]
    NODE,
    #[regex(r"\*ELEMENT[^\*]*")]
    ELEMENT,
    #[regex(r"\*END[^\*]*")]
    END,

    GEOMETRY, // as long as len don't change, shouldn't bother
    CARD,     // keyword + deck
    KEYWORD,  // *PART
    RECORD,   // every keyword at least follows one record
    RECORDS,  // 0 or many, not one
    DECK,     // RECORD + RECORDS

    #[error]
    ERROR,
    ROOT,
    EOF,
}

impl SyntaxKind {
    /// enum var back to u16
    fn discriminant(&self) -> u16 {
        unsafe { *(self as *const Self as *const u16) }
    }

    #[inline]
    pub fn is_trivia(self) -> bool {
        matches!(self, SyntaxKind::COMMENT | SyntaxKind::GEOMETRY)
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind.discriminant())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lang {}
impl rowan::Language for Lang {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= SyntaxKind::ROOT.discriminant());
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<Lang>;
#[allow(unused)]
pub type SyntaxToken = rowan::SyntaxToken<Lang>;
#[allow(unused)]
pub type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

pub type SyntaxNodeChildren = rowan::SyntaxNodeChildren<Lang>;
pub type SyntaxElementChildren = rowan::SyntaxElementChildren<Lang>;
pub type PreorderWithTokens = rowan::api::PreorderWithTokens<Lang>;

#[derive(Default)]
///TODO:!
pub struct SyntaxTreeBuilder {
    errors: Vec<SyntaxError>,
    inner: GreenNodeBuilder<'static>,
}

impl SyntaxTreeBuilder {
    pub(crate) fn finish_raw(self) -> (GreenNode, Vec<SyntaxError>) {
        let green = self.inner.finish();
        (green, self.errors)
    }

    pub fn token(&mut self, kind: SyntaxKind, text: &str) {
        let kind = Lang::kind_to_raw(kind);
        self.inner.token(kind, text);
    }

    pub fn start_node(&mut self, kind: SyntaxKind) {
        let kind = Lang::kind_to_raw(kind);
        self.inner.start_node(kind);
    }

    pub fn finish_node(&mut self) {
        self.inner.finish_node();
    }

    pub fn error(&mut self, error: String, text_pos: TextSize) {
        self.errors
            .push(SyntaxError::new_at_offset(error, text_pos));
    }
}

// print a node to std
pub fn print(indent: usize, element: SyntaxElement) {
    let kind: SyntaxKind = element.kind().into();
    print!("{:indent$}", "", indent = indent);
    match element {
        NodeOrToken::Node(node) => {
            println!("- {:?}", kind);
            for child in node.children_with_tokens() {
                print(indent + 2, child);
            }
        }

        NodeOrToken::Token(token) => println!("- {:?} {:?}", token.text(), kind),
    }
}
