//! Function definitions
//!
//! While I want to eventually add support for a bunch of syntax sugar and
//! things like optional/keyword arguments, for now it just supports the
//! simplest case to get started.

use crate::lexer::{Delimiter, TokenKind};

use super::*;

/// Function definitions.
///
/// # Grammar
///
/// Function := '(' sep_by_trailing(Parameter, ',') ')' '=>' Expression
#[derive(Debug)]
pub struct Function<'a> {
    open: Span,
    parameters: Vec<Parameter<'a>>,
    commas: Vec<Span>,
    close: Span,
    arrow: Span,
    body: Box<Expression<'a>>,
}

impl<'a> Function<'a> {
    /// Get a reference to the function's parameter list.
    pub fn parameters(&self) -> &[Parameter<'a>] {
        &self.parameters
    }

    /// The span for the commas that were between parameters, in order.
    pub fn comma(&self) -> &[Span] {
        &self.commas
    }

    /// Get a reference to the function's body expression.
    pub fn body(&self) -> &Expression {
        self.body.as_ref()
    }

    /// The span for the opening parenthesis token.
    pub fn open(&self) -> Span {
        self.open
    }

    /// The span fo the closing parenthesis token.
    pub fn close(&self) -> Span {
        self.close
    }

    /// The span of the double arrow (`=>`) token.
    pub fn arrow(&self) -> Span {
        self.arrow
    }
}

impl<'a> Parse<'a> for Function<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Self, Error> {
        let open = parser
            .consume(TokenKind::Open(Delimiter::Parenthesis), Self::NAME)?
            .span();

        let (parameters, commas) = parser.sep_by_trailing(TokenKind::Comma)?;

        let close = parser
            .consume(TokenKind::Close(Delimiter::Parenthesis), Self::NAME)?
            .span();

        let arrow = parser.consume(TokenKind::DoubleArrow, Self::NAME)?.span();

        let body = Box::new(parser.parse()?);

        Ok(Function {
            open,
            parameters,
            commas,
            close,
            arrow,
            body,
        })
    }
}

impl<'a> Syntax for Function<'a> {
    const NAME: &'static str = "a function";

    fn span(&self) -> Span {
        self.open + self.body.span()
    }
}

#[derive(Debug)]
pub struct Parameter<'a> {
    name: Identifier<'a>,
}

impl<'a> Parameter<'a> {
    /// The name of the parameter
    pub fn name(&self) -> &Identifier<'a> {
        &self.name
    }
}

impl<'a> Parse<'a> for Parameter<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Self, Error> {
        parser
            .parse()
            .map(|name| Parameter { name })
            .map_err(|e| e.set_wanted("function parameter name"))
    }
}

impl<'a> Syntax for Parameter<'a> {
    const NAME: &'static str = "a parameter";

    fn span(&self) -> Span {
        self.name.span()
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_parameter() {
        let mut parser = Parser::new(" foo ").unwrap();
        assert!(parser.parse::<Parameter>().is_ok());
        assert!(parser.is_empty());
    }

    #[test]
    fn test_function() {
        let mut parser = Parser::new(" (x) => 1").unwrap();
        let result = parser.parse::<Function>();
        assert!(result.is_ok(), "failed with {:?}", result);
        assert!(parser.is_empty(),);
    }

    #[test]
    fn test_function_params() {
        let mut parser = Parser::new(" (x, y, z ) => 1").unwrap();
        let result = parser.parse::<Function>();
        assert!(result.is_ok(), "failed with {:?}", result);
        assert!(parser.is_empty(),);
    }

    #[test]
    fn test_function_trailing() {
        let mut parser = Parser::new(" (x, ) => 1").unwrap();
        let result = parser.parse::<Function>();
        assert!(result.is_ok(), "failed with {:?}", result);
        assert!(parser.is_empty(),);
    }

    #[test]
    fn test_function_nested() {
        let mut parser = Parser::new("(a) => (b) => c").unwrap();
        let result = parser.parse::<Function>();
        assert!(result.is_ok(), "failed with {:?}", result);
        assert!(parser.is_empty(),);
    }
}
