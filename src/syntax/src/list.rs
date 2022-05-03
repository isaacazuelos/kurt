//! List are sequences of values

use crate::lexer::{Delimiter, TokenKind};

use super::*;

/// List literals.
///
/// A comma-delimited sequence of expressions between square brackets.
///
/// # Grammar
///
/// [`List`] := `[` [`sep_by_trailing`][1]([`Expression`], `,`) `]`
///
/// [1]: Parser::sep_by_trailing
#[derive(Debug)]
pub struct List<'a> {
    open: Span,
    elements: Vec<Expression<'a>>,
    commas: Vec<Span>,
    close: Span,
}

impl<'a> List<'a> {
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
    type SyntaxError = SyntaxError;

    fn parse_with(parser: &mut Parser<'a>) -> SyntaxResult<Self> {
        let open = parser
            .consume(TokenKind::Open(Delimiter::Bracket))
            .ok_or_else(|| SyntaxError::ListNoOpen(parser.peek_span()))?
            .span();

        let (elements, commas) = parser.sep_by_trailing(TokenKind::Comma)?;

        let close = parser
            .consume(TokenKind::Close(Delimiter::Bracket))
            .ok_or_else(|| SyntaxError::ListNoClose(open, parser.peek_span()))?
            .span();

        Ok(List {
            open,
            elements,
            commas,
            close,
        })
    }
}

impl<'a> Sequence for List<'a> {
    type Element = Expression<'a>;

    const SEPARATOR: TokenKind = TokenKind::Comma;

    fn elements(&self) -> &[Self::Element] {
        &self.elements
    }

    fn separators(&self) -> &[Span] {
        &self.commas
    }
}

impl<'a> Syntax for List<'a> {
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
