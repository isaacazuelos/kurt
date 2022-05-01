//! Subscripts
//!
//! Accessing values using another value as a keys.

use diagnostic::Span;

use parser::lexer::{Delimiter, TokenKind};

use crate::*;

/// Subscript indexing
///
/// See the note on [`Call`] about why don't implement [`Parse`].
///
/// # Grammar
///
/// [`Subscript`] := [`Expression`] `[` [`Expression`] `]`
#[derive(Debug)]
pub struct Subscript<'a> {
    target: Box<Expression<'a>>,
    open: Span,
    index: Box<Expression<'a>>,
    close: Span,
}

impl<'a> Subscript<'a> {
    /// Get a reference to the subscript's target, the thing we're indexing.
    pub fn target(&self) -> &Expression<'a> {
        &self.target
    }

    /// The span of the subscripts's open bracket.
    pub fn open(&self) -> Span {
        self.open
    }

    /// Get a reference to the expression that we're indexing by.
    pub fn index(&self) -> &Expression<'a> {
        self.index.as_ref()
    }

    /// The span of the subscripts's close bracket.
    pub fn close(&self) -> Span {
        self.close
    }
}

impl<'a> Syntax for Subscript<'a> {
    fn span(&self) -> Span {
        self.target.span() + self.close
    }
}

impl<'a> Subscript<'a> {
    /// Parse a single subscript, starting with an already-parsed given primary
    /// 'target' for the subscript.
    pub(crate) fn parse_from(
        target: Expression<'a>,
        parser: &mut Parser<'a>,
    ) -> SyntaxResult<Self> {
        let open = parser
            .consume(TokenKind::Open(Delimiter::Bracket))
            .ok_or(SyntaxError::SubscriptNoOpen)?
            .span();

        let index = Box::new(parser.parse()?);

        let close = parser
            .consume(TokenKind::Close(Delimiter::Bracket))
            .ok_or(SyntaxError::SubscriptNoClose)?
            .span();

        Ok(Subscript {
            target: Box::new(target),
            open,
            index,
            close,
        })
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    // We use the Expression::parse since we don't implement Parse. See the note
    // on [`Subscript`].

    #[test]
    fn subscript_empty() {
        let mut parser = Parser::new(" foo[] ").unwrap();
        let result = parser.parse::<Expression>();
        assert!(result.is_err(), "succeeded with {:?}", result);
    }

    #[test]
    fn subscript_normal() {
        let mut parser = Parser::new(" foo[1] ").unwrap();
        let result = parser.parse::<Expression>();
        assert!(result.is_ok(), "failed with with {:?}", result);
        assert!(parser.is_empty());
    }

    #[test]
    fn subscript_chain() {
        let mut parser = Parser::new(" foo[1][2] ").unwrap();
        let result = parser.parse::<Expression>();
        assert!(result.is_ok(), "failed with with {:?}", result);
        assert!(parser.is_empty());
    }
}
