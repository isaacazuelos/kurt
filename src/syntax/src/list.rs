//! List are sequences of values
//!
//! A comma-delimited sequence of expressions between square brackets.

use crate::lexer::{Delimiter, TokenKind};

use super::*;

/// List literals
///
/// # Grammar
///
///
/// List := '[' sep_by_trailing(Expression, ',') ']'
#[derive(Debug)]
pub struct List<'a> {
    open: Span,
    elements: Vec<Expression<'a>>,
    commas: Vec<Span>,
    close: Span,
}

impl<'a> List<'a> {
    /// Get a slice containing the elements of the list.
    pub fn elements(&self) -> &[Expression<'a>] {
        &self.elements
    }

    /// Get a slice with the span for each comma in the list
    pub fn commas(&self) -> &[Span] {
        &self.commas
    }

    /// The span for the opening bracket token.
    pub fn open(&self) -> Span {
        self.open
    }

    /// The span fo the closing bracket token.
    pub fn close(&self) -> Span {
        self.close
    }
}

impl<'a> Parse<'a> for List<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Self, Error> {
        let open = parser
            .consume(TokenKind::Open(Delimiter::Bracket), Self::NAME)?
            .span();

        let (elements, commas) = parser.sep_by_trailing(TokenKind::Comma)?;

        let close = parser
            .consume(TokenKind::Close(Delimiter::Bracket), Self::NAME)?
            .span();

        Ok(List {
            open,
            elements,
            commas,
            close,
        })
    }
}

impl<'a> Syntax for List<'a> {
    const NAME: &'static str = "a list";

    fn span(&self) -> Span {
        self.open + self.close()
    }
}

#[cfg(test)]
mod list_tests {
    use super::*;

    #[test]
    fn empty() {
        let mut parser = Parser::new(" [] ").unwrap();
        let result = parser.parse::<List>();
        assert!(result.is_ok());
        assert!(parser.is_empty());
    }

    #[test]
    fn only_trailing_comma() {
        let mut parser = Parser::new(" [ , ] ").unwrap();
        let result = parser.parse::<List>();
        assert!(result.is_ok());
        assert!(parser.is_empty());
    }

    #[test]
    fn element() {
        let mut parser = Parser::new(" [ 1 ] ").unwrap();
        let result = parser.parse::<List>();
        assert!(result.is_ok());
        assert!(parser.is_empty());
    }

    #[test]
    fn multiple_elements() {
        let mut parser = Parser::new(" [ 1, 2, 3 ] ").unwrap();
        let result = parser.parse::<List>();
        assert!(result.is_ok());
        assert!(parser.is_empty());
    }

    #[test]
    fn trailing_comma() {
        let mut parser = Parser::new(" [ 1, ] ").unwrap();
        let result = parser.parse::<List>();
        assert!(result.is_ok());
        assert!(parser.is_empty());
    }
}
