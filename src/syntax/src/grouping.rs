//! Using parenthesis to group up expressions.

use crate::lexer::{Delimiter, TokenKind};

use super::*;

/// Parentheses used for groupings.
///
/// # Grammar
///
/// [`Grouping`] := `(` [`Expression`] `)`
#[derive(Debug)]
pub struct Grouping<'a> {
    open: Span,
    inner: Box<Expression<'a>>,
    close: Span,
}

impl<'a> Grouping<'a> {
    /// The span of the opening parenthesis token.
    pub fn open(&self) -> Span {
        self.open
    }

    /// Get the inner expression.
    pub fn body(&self) -> &Expression {
        self.inner.as_ref()
    }

    /// The span of the closing parenthesis token.
    pub fn close(&self) -> Span {
        self.close
    }
}

impl<'a> Syntax for Grouping<'a> {
    fn span(&self) -> Span {
        self.open + self.close
    }
}

impl<'a> Parse<'a> for Grouping<'a> {
    type SyntaxError = SyntaxError;

    fn parse_with(parser: &mut Parser<'a>) -> SyntaxResult<Self> {
        let open = parser
            .consume(TokenKind::Open(Delimiter::Parenthesis))
            .ok_or_else(|| SyntaxError::GroupingNoOpen(parser.peek_span()))?
            .span();

        let body = parser.parse()?;

        let close = parser
            .consume(TokenKind::Close(Delimiter::Parenthesis))
            .ok_or_else(|| {
                SyntaxError::GroupingNoClose(open, parser.peek_span())
            })?
            .span();

        Ok(Grouping {
            open,
            inner: Box::new(body),
            close,
        })
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn parse_grouping() {
        let mut parser = Parser::new("(1)").unwrap();
        let result = parser.parse::<Grouping>();
        assert!(result.is_ok(), "failed with {:?}", result);
    }

    #[test]
    fn parse_grouping_nested() {
        let mut parser = Parser::new("((1))").unwrap();
        let result = parser.parse::<Grouping>();
        assert!(result.is_ok(), "failed with {:?}", result);
    }
}
