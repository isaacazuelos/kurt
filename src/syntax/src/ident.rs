//! Identifiers

use diagnostic::Span;
use unicode_normalization::UnicodeNormalization;

use parser::{
    lexer::{Token, TokenKind},
    Error, Parse, Parser,
};

use crate::Syntax;

/// Identifiers
///
/// This does UTF8 normalization so that consumers of the AST can compare
/// identifiers.
///
/// # Grammar
///
/// [`Identifier`] := [`TokenKind::Identifier`]
#[derive(Debug)]
pub struct Identifier {
    body: String,
    span: Span,
}

impl Identifier {
    /// Create a new identifier from a token.
    fn new(token: Token) -> Identifier {
        Identifier {
            body: token.body().nfc().collect(),
            span: token.span(),
        }
    }

    /// View the identifier as a `&str`.
    pub fn as_str(&self) -> &str {
        &self.body
    }
}

impl<'a> Syntax for Identifier {
    const NAME: &'static str = "an identifier";

    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> Parse<'a> for Identifier {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Identifier, Error> {
        parser
            .consume(TokenKind::Identifier, Self::NAME)
            .map(Identifier::new)
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_identifier() {
        let mut parser = Parser::new("hello").unwrap();
        assert!(parser.parse::<Identifier>().is_ok());
        assert!(parser.is_empty());
    }
}
