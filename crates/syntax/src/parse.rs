use std::{marker::PhantomData, sync::Arc};

use logos::{Logos, Span};
pub use rowan::{GreenNode, TextRange, TextSize};

use syntax_node::SyntaxKind;

use SyntaxKind::*;

use crate::{
    ast::AstNode,
    dyna_nodes::SourceFile,
    syntax_error::SyntaxError,
    syntax_node::{self, SyntaxNode, SyntaxTreeBuilder},
};

#[derive(Debug, PartialEq, Eq)]
pub struct Parse<T> {
    pub green: GreenNode,
    pub errors: Arc<Vec<SyntaxError>>,
    _ty: PhantomData<fn() -> T>,
}

impl<T> Clone for Parse<T> {
    fn clone(&self) -> Parse<T> {
        Parse {
            green: self.green.clone(),
            errors: self.errors.clone(),
            _ty: PhantomData,
        }
    }
}

impl<T> Parse<T> {
    fn new(green: GreenNode, errors: Vec<SyntaxError>) -> Parse<T> {
        Parse {
            green,
            errors: Arc::new(errors),
            _ty: PhantomData,
        }
    }

    pub fn syntax_node(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }
    pub fn errors(&self) -> &[SyntaxError] {
        &self.errors
    }
}

impl<T: AstNode> Parse<T> {
    pub fn to_syntax(self) -> Parse<SyntaxNode> {
        Parse {
            green: self.green,
            errors: self.errors,
            _ty: PhantomData,
        }
    }

    // ast node cast!
    pub fn tree(&self) -> T {
        T::cast(self.syntax_node()).expect("should cast node")
    }

    pub fn ok(self) -> Result<T, Arc<Vec<SyntaxError>>> {
        if self.errors.is_empty() {
            Ok(self.tree())
        } else {
            Err(self.errors)
        }
    }
}

impl Parse<SyntaxNode> {
    pub fn cast<N: AstNode>(self) -> Option<Parse<N>> {
        if N::cast(self.syntax_node()).is_some() {
            Some(Parse {
                green: self.green,
                errors: self.errors,
                _ty: PhantomData,
            })
        } else {
            None
        }
    }
}

// actually only used in a function
pub struct CstParser<'a> {
    text: &'a str,
    /// input tokens, including whitespace,
    /// in *reverse* order.
    tokens: Vec<(SyntaxKind, TextRange)>,
    /// the in-progress tree.
    builder: SyntaxTreeBuilder,
}

impl<'a> CstParser<'a> {
    /// text should live longer than this parser
    pub fn new<'b: 'a>(text: &'b str) -> CstParser<'a> {
        let range = |span: Span| {
            TextRange::new(span.start.try_into().unwrap(), span.end.try_into().unwrap())
        };
        let mut tokens: Vec<_> = SyntaxKind::lexer(text)
            .spanned()
            .map(|i| (i.0, range(i.1)))
            .collect();
        // so pop return first
        tokens.reverse();
        Self {
            text,
            tokens,
            builder: SyntaxTreeBuilder::default(),
        }
    }

    fn node_from_line_a(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind);
        self.skip_comment();
        loop {
            match self.current() {
                Some(NEWLINE) => {
                    self.bump();
                    break;
                }
                None => {
                    // // eof error
                    // let err = String::from("reaching EOF too soon");
                    // let pos = self.text.len();
                    // self.builder.error(err, pos.try_into().unwrap());
                    break;
                }
                _ => self.bump(),
            }
        }
        self.builder.finish_node();
    }

    fn records(&mut self) {
        self.builder.start_node(RECORDS);
        self.skip_comment();
        loop {
            match self.current() {
                Some(ASTERISK) | None | Some(NODE) | Some(ELEMENT) | Some(END) => {
                    break;
                }
                _ => self.bump(),
            }
        }
        self.builder.finish_node();
    }

    fn card(&mut self) {
        debug_assert!(self.current() == Some(ASTERISK));
        self.builder.start_node(CARD); // 1
        self.skip_comment();
        self.node_from_line_a(KEYWORD);
        if self.current() == Some(ASTERISK) {
            self.builder.error(
                "new card in card!".to_string(),
                self.tokens.last().unwrap().1.start(),
            );
            self.builder.finish_node();
            self.card(); // little recurse
            return;
        }
        self.builder.start_node(DECK); // 2
        self.skip_comment();
        self.node_from_line_a(RECORD);
        self.skip_comment();
        self.records();
        self.builder.finish_node();
        self.builder.finish_node();
    }

    pub fn parse(mut self) -> Parse<SourceFile> {
        self.builder.start_node(ROOT);
        self.skip_comment();
        loop {
            match self.current() {
                Some(ASTERISK) => {
                    self.card();
                }
                None | Some(END) => break,
                Some(NODE) | Some(ELEMENT) => {
                    self.builder.start_node(GEOMETRY);
                    self.bump();
                    self.builder.finish_node();
                }
                _ => {
                    let err = String::from("what is?");
                    let pos = self.tokens.last().unwrap().1.start();
                    self.builder.error(err, pos);
                    self.builder.start_node(ERROR);
                    self.bump();
                    self.builder.finish_node();
                }
            }
        }
        self.builder.finish_node();
        let (node, errors) = self.builder.finish_raw();
        let errors = Arc::new(errors);
        Parse {
            green: node,
            errors,
            _ty: PhantomData,
        }
    }

    fn skip_some(&mut self, some: &[SyntaxKind]) {
        while some.contains(&self.current().unwrap_or(EOF)) {
            self.bump()
        }
    }

    fn skip_comment(&mut self) {
        self.skip_some(&[COMMENT])
    }

    // so that's what you call lossless, it caches str
    fn bump(&mut self) {
        let (kind, range) = self.tokens.pop().unwrap();
        self.builder.token(kind.into(), &self.text[range]);
    }

    fn current(&self) -> Option<SyntaxKind> {
        self.tokens.last().map(|(kind, _)| *kind)
    }
}

// my first play with phantom data
pub fn parse_text(text: &str) -> Parse<SourceFile> {
    let parser = CstParser::new(text);
    parser.parse()
}
