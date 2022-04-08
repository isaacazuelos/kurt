//! Syntax for literal values.

use diagnostic::Span;

use parser::{lexer::TokenKind, Error, Parse, Parser};

use crate::Syntax;

/// The different kinds of literal values that can appear in source code.
///
/// These aren't quite the same as types, since both `0` an `0x0` produce the
/// same value at runtime and could be the same type, but aren't the same kind
/// of literal value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Binary,
    Bool,
    Char,
    Decimal,
    Float,
    Hexadecimal,
    Keyword,
    Octal,
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
    const NAME: &'static str = "a value literal";

    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> Parse<'a> for Literal<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Literal<'a>, Error> {
        use Kind as LiteralKind; // to keep them clearer

        let kind = match parser.peek() {
            Some(TokenKind::Bool) => LiteralKind::Bool,
            Some(TokenKind::Char) => LiteralKind::Char,
            Some(TokenKind::Bin) => LiteralKind::Binary,
            Some(TokenKind::Hex) => LiteralKind::Hexadecimal,
            Some(TokenKind::Int) => LiteralKind::Decimal,
            Some(TokenKind::Oct) => LiteralKind::Octal,
            Some(TokenKind::Float) => LiteralKind::Float,
            Some(TokenKind::String) => LiteralKind::String,
            Some(TokenKind::Colon) => LiteralKind::Keyword,
            Some(found) => {
                return Err(Error::Unexpected {
                    wanted: Self::NAME,
                    found,
                })
            }
            None => return Err(Error::EOFExpecting(Self::NAME)),
        };

        let token = parser.advance().unwrap();

        if kind == LiteralKind::Keyword {
            match parser.peek() {
                Some(TokenKind::Identifier) => {
                    let id = parser.advance().unwrap();

                    if token.span().end() == id.span().start() {
                        let span = token.span() + id.span();
                        Ok(Literal::new(kind, id.body(), span))
                    } else {
                        Err(Error::KeywordNoSpace)
                    }
                }

                Some(found) => Err(Error::Unexpected {
                    wanted: Self::NAME,
                    found,
                }),
                None => Err(Error::EOFExpecting(Self::NAME)),
            }
        } else {
            Ok(Literal::new(kind, token.body(), token.span()))
        }
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

    #[test]
    fn parse_keyword() {
        let mut parser = Parser::new(" :hello_world ").unwrap();
        let literal = parser.parse::<Literal>().unwrap();
        assert_eq!(literal.kind(), Kind::Keyword);
        assert_eq!(literal.body(), "hello_world");
        assert!(parser.is_empty());
    }

    #[test]
    fn parse_keyword_no_identifier() {
        let mut parser = Parser::new(" : ").unwrap();
        let literal = parser.parse::<Literal>();
        assert!(literal.is_err());
    }

    #[test]
    fn parse_keyword_non_identifier() {
        let mut parser = Parser::new(" :1").unwrap();
        let literal = parser.parse::<Literal>();
        assert!(literal.is_err());
    }

    #[test]
    fn parse_keyword_space() {
        let mut parser = Parser::new(" : hi ").unwrap();
        let literal = parser.parse::<Literal>();
        assert!(literal.is_err());
    }
}
