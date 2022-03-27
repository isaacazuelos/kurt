//! Identifiers

use crate::lexer::{Token, TokenKind};

use super::*;

#[derive(Debug)]
pub struct Identifier<'a> {
    token: Token<'a>,
}

impl<'a> Syntax for Identifier<'a> {
    fn span(&self) -> Span {
        self.token.span()
    }
}

impl<'a> Parse<'a> for Identifier<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Identifier<'a>, Error> {
        let token = parser.advance().unwrap();

        match token.kind() {
            TokenKind::Identifier => Ok(Identifier { token }),
            found => Err(Error::Unexpected {
                wanted: TokenKind::Identifier,
                found,
            }),
        }
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
