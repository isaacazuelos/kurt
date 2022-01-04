//! Syntax for literal values.

use diagnostic::Span;

use crate::{
    ast,
    lexer::TokenKind,
    parser::{Error, Parser},
};

use super::{Parse, Syntax};

/// The different kinds of literal values that can appear in source code.
///
/// These aren't quite the same as types, since both `0` an `0x0` produce the
/// same value at runtime and could be the same type, but aren't the same kind
/// of literal value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Bool,
    Char,
    Decimal,
    Hexadecimal,
    Octal,
    Binary,
    Float,
    String,
    Unit,
}

/// A literal value is something like `123` or `false` which produces a specific
/// value at runtime.
#[derive(Debug)]
pub struct Literal<'a> {
    kind: Kind,
    body: &'a str,
    span: Span,
}

impl<'a> Literal<'a> {
    /// Create a new literal value.
    ///
    /// This will allocate to store the `body`
    pub fn new(kind: Kind, body: &'a str, span: Span) -> Literal<'a> {
        Literal { kind, body, span }
    }

    /// The [`Kind`] of literal value this is.
    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// The text from the source for this literal value.
    pub fn body(&self) -> &str {
        self.body
    }
}

impl<'a> Syntax for Literal<'a> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> Parse<'a> for Literal<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Literal<'a>, Error> {
        let token = parser
            .advance()
            .expect("Parser::literal expected a literal token");

        let kind = match token.kind() {
            TokenKind::Bool => ast::LiteralKind::Bool,
            TokenKind::Char => ast::LiteralKind::Char,
            TokenKind::Bin => ast::LiteralKind::Binary,
            TokenKind::Hex => ast::LiteralKind::Hexadecimal,
            TokenKind::Int => ast::LiteralKind::Decimal,
            TokenKind::Oct => ast::LiteralKind::Octal,
            TokenKind::Float => ast::LiteralKind::Float,
            TokenKind::String => ast::LiteralKind::String,
            k => unreachable!("Token::is_literal and Parser::literal disagrees about {:?} being a literal", k),
        };

        Ok(Literal::new(kind, token.body(), token.span()))
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn parse_literal() {
        let mut parser = Parser::new("0 a").unwrap();
        let literal = parser.parse::<Literal>();
        assert!(literal.is_ok());
        assert_eq!(literal.unwrap().kind(), Kind::Decimal);

        assert!(!parser.is_empty());
    }

    #[test]
    fn parse_literal_string() {
        let mut parser = Parser::new(" \"Hello, world!\\n\" ").unwrap();
        let literal = parser.parse::<Literal>().unwrap();
        assert_eq!(literal.kind(), Kind::String);
        assert_eq!(literal.body(), "\"Hello, world!\\n\"");
        assert!(parser.is_empty());
    }
}
