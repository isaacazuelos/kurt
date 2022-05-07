//! Syntax for literal values.

use diagnostic::Span;

use parser::{
    lexer::{Delimiter, TokenKind},
    Error, Parse, Parser,
};

use crate::*;

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
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> Parse<'a> for Literal<'a> {
    type SyntaxError = SyntaxError;

    fn parse_with(parser: &mut Parser<'a>) -> SyntaxResult<Literal<'a>> {
        match parser.peek_kind() {
            Some(TokenKind::Colon) => Literal::parse_keyword_with(parser),
            Some(TokenKind::Open(Delimiter::Parenthesis)) => {
                Literal::parse_unit(parser)
            }
            _ => Literal::parse_non_keyword_with(parser),
        }
    }
}

impl<'a> Literal<'a> {
    fn parse_unit(parser: &mut Parser<'a>) -> SyntaxResult<Literal<'a>> {
        let open = parser
            .consume(TokenKind::Open(Delimiter::Parenthesis))
            .ok_or_else(|| SyntaxError::UnitNoOpen(parser.next_span()))?;

        let close = parser
            .consume(TokenKind::Close(Delimiter::Parenthesis))
            .ok_or_else(|| {
                SyntaxError::UnitNoClose(open.span(), parser.next_span())
            })?;

        Ok(Literal::new(Kind::Unit, "", open.span() + close.span()))
    }

    fn parse_keyword_with(
        parser: &mut Parser<'a>,
    ) -> SyntaxResult<Literal<'a>> {
        let colon = parser
            .consume(TokenKind::Colon)
            .ok_or_else(|| SyntaxError::KeywordNoColon(parser.next_span()))?;

        let name = parser.consume(TokenKind::Identifier).ok_or_else(|| {
            SyntaxError::KeywordNoName(colon.span(), parser.next_span())
        })?;

        // Check to rule out keyword like `: foo`
        if colon.span().end() != name.span().start() {
            return Err(
                SyntaxError::KeywordNoSpace(colon.span(), name.span()).into()
            );
        }

        Ok(Literal::new(
            Kind::Keyword,
            name.body(),
            colon.span() + name.span(),
        ))
    }

    fn parse_non_keyword_with(
        parser: &mut Parser<'a>,
    ) -> SyntaxResult<Literal<'a>> {
        let token = parser
            .consume_if(|token| token.kind().is_literal())
            .ok_or_else(|| Error::EOF(parser.eof_span()))?;

        let kind = match token.kind() {
            TokenKind::Bool => Kind::Bool,
            TokenKind::Char => Kind::Char,
            TokenKind::Bin => Kind::Binary,
            TokenKind::Hex => Kind::Hexadecimal,
            TokenKind::Int => Kind::Decimal,
            TokenKind::Oct => Kind::Octal,
            TokenKind::Float => Kind::Float,
            TokenKind::String => Kind::String,
            _ => unreachable!("ruled out by is_literal in consume_if"),
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

    #[test]
    fn parse_unit() {
        let mut parser = Parser::new(" () ").unwrap();
        let literal = parser.parse::<Literal>();
        assert!(literal.is_ok());
    }
}
