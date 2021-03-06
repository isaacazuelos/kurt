//! Function calls, like `foo(1, 2)`

use diagnostic::Span;

use parser::lexer::{Delimiter, TokenKind};

use super::*;

/// Function calls.
///
/// We don't have a lot of fancy features yet -- no keyword or optional
/// arguments.
///
/// # Grammar
///
/// [`Call`] := Expression `(` [`sep_by_trailing`][1]([`Expression`], `,`) `)`
///
/// Note that [`Call`] doesn't implement [`Parse`]. This is is because we can't
/// reasonably know the 'most greedy' parse of a call without mixing in
/// different kinds of postfix expressions. For example `f()[0]()` should use
/// all the input. We'd have to parse any ['primary'][1] expression, and then
/// unpack backwards to find the outer-most call, but we can't do that without
/// messing up the parser state.
///
/// [1]: crate::Expression::primary
#[derive(Debug)]
pub struct Call<'a> {
    target: Box<Expression<'a>>,
    open: Span,
    arguments: Vec<Expression<'a>>,
    commas: Vec<Span>,
    close: Span,
}

impl<'a> Call<'a> {
    /// Get a reference to the call's target.
    pub fn target(&self) -> &Expression<'a> {
        &self.target
    }

    /// The span of the call's open parenthesis.
    pub fn open(&self) -> Span {
        self.open
    }

    /// The span of the call's close parenthesis.
    pub fn close(&self) -> Span {
        self.close
    }
}

impl<'a> Syntax for Call<'a> {
    fn span(&self) -> Span {
        self.target.span() + self.close
    }
}

impl<'a> Sequence for Call<'a> {
    type Element = Expression<'a>;

    const SEPARATOR: TokenKind = TokenKind::Comma;

    fn elements(&self) -> &[Self::Element] {
        &self.arguments
    }

    fn separators(&self) -> &[Span] {
        &self.commas
    }
}

impl<'a> Call<'a> {
    /// Parse a single call, starting with an already-parsed given primary
    /// 'target' for the call.
    pub(crate) fn parse_from(
        target: Expression<'a>,
        parser: &mut Parser<'a>,
    ) -> SyntaxResult<Self> {
        let open = parser
            .consume(TokenKind::Open(Delimiter::Parenthesis))
            .ok_or_else(|| {
                SyntaxError::CallNoOpen(target.span(), parser.next_span())
            })?
            .span();

        let (arguments, commas) = parser.sep_by_trailing(TokenKind::Comma)?;

        let close = parser
            .consume(TokenKind::Close(Delimiter::Parenthesis))
            .ok_or_else(|| SyntaxError::CallNoClose(open, parser.next_span()))?
            .span();

        Ok(Call {
            target: Box::new(target),
            open,
            arguments,
            commas,
            close,
        })
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    // We use the Expression::parse since we don't implement Parse. See the note
    // on [`Call`].

    #[test]
    fn call_empty() {
        let mut parser = Parser::new(" foo() ").unwrap();
        let result = parser.parse::<Expression>();
        assert!(result.is_ok(), "failed with {:?}", result);
        assert!(parser.is_empty());
    }

    #[test]
    fn call_arg() {
        let mut parser = Parser::new(" foo(1) ").unwrap();
        assert!(parser.parse::<Expression>().is_ok());
        assert!(parser.is_empty());
    }

    #[test]
    fn call_arg_trailing() {
        let mut parser = Parser::new(" foo(1, 2, 3, ) ").unwrap();
        assert!(parser.parse::<Expression>().is_ok());
        assert!(parser.is_empty());
    }

    #[test]
    fn call_nested() {
        let mut parser = Parser::new(" foo(bar()) ").unwrap();
        let result = parser.parse::<Expression>();
        assert!(result.is_ok(), "expected call but got {:?}", result);
        assert!(parser.is_empty());
    }

    #[test]
    fn call_curry() {
        let mut parser = Parser::new(" foo(1)(2) ").unwrap();
        let result = parser.parse::<Expression>();
        assert!(result.is_ok(), "expected call but got {:?}", result);
        assert!(parser.is_empty());
    }
}
