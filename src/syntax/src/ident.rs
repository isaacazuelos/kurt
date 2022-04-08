//! Identifiers

// TODO: UTF-8 Normalization?

use diagnostic::Span;

use parser::{
    lexer::{Token, TokenKind},
    Error, Parse, Parser,
};

use crate::Syntax;

#[derive(Debug)]
pub struct Identifier<'a> {
    token: Token<'a>,
}

impl<'a> Identifier<'a> {
    /// View the identifier as a `&str`.
    pub fn as_str(&'a self) -> &'a str {
        self.token.body()
    }
}

impl<'a> Syntax for Identifier<'a> {
    const NAME: &'static str = "an identifier";

    fn span(&self) -> Span {
        self.token.span()
    }
}

impl<'a> Parse<'a> for Identifier<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Identifier<'a>, Error> {
        parser
            .consume(TokenKind::Identifier, Self::NAME)
            .map(|token| Identifier { token })
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
