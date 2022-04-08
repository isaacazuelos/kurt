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
/// [`Function`] := [`(`][`TokenKind::Open`] [`ParameterList`] [`)`][`TokenKind::Close`] [`=>`][TokenKind::DoubleArrow] [`Expression`]
#[derive(Debug)]
pub struct Function<'a> {
    open: Span,
    parameters: ParameterList<'a>,
    close: Span,
    arrow: Span,
    body: Box<Expression<'a>>,
}

impl<'a> Function<'a> {
    /// Get a reference to the function's parameter list.
    pub fn parameters(&self) -> &ParameterList<'a> {
        &self.parameters
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

        let parameters = parser.parse()?;

        let close = parser
            .consume(TokenKind::Close(Delimiter::Parenthesis), Self::NAME)?
            .span();

        let arrow = parser.consume(TokenKind::DoubleArrow, Self::NAME)?.span();

        let body = Box::new(parser.parse()?);

        Ok(Function {
            open,
            parameters,
            close,
            arrow,
            body,
        })
    }
}

impl<'a> Syntax for Function<'a> {
    const NAME: &'static str = "a function";

    fn span(&self) -> Span {
        self.parameters.span() + self.body.span()
    }
}

/// # Grammar
///
/// [`ParameterList`] := [`sep_by(`][`Parser::sep_by_trailing`] [`ParameterList`] [`)`][`TokenKind::Close`] [`=>`][TokenKind::DoubleArrow] [`Expression`]
#[derive(Debug)]
pub struct ParameterList<'a> {
    parameters: Vec<Parameter<'a>>,
    commas: Vec<Span>,
}

impl<'a> ParameterList<'a> {
    /// Get a reference to the parameter list's parameters.
    pub fn parameters(&self) -> &[Parameter] {
        self.parameters.as_ref()
    }

    /// Get a reference to the parameter list's commas.
    pub fn commas(&self) -> &[Span] {
        self.commas.as_ref()
    }
}

impl<'a> Parse<'a> for ParameterList<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Self, Error> {
        parser
            .sep_by_trailing(TokenKind::Comma)
            .map(|(parameters, commas)| ParameterList { parameters, commas })
    }
}

impl<'a> Syntax for ParameterList<'a> {
    const NAME: &'static str = "a function's parameter list";

    fn span(&self) -> Span {
        if let Some(first) = self.parameters.first() {
            first.span() + self.parameters.last().unwrap().span()
        } else {
            Span::default()
        }
    }
}

#[derive(Debug)]
pub struct Parameter<'a> {
    name: Identifier<'a>,
}

impl<'a> Parameter<'a> {}

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
    fn test_parameter_list() {
        let mut parser = Parser::new(" ").unwrap();
        assert!(parser.parse::<ParameterList>().is_ok());
        assert!(parser.is_empty());

        let mut parser = Parser::new(" foo ").unwrap();
        assert!(parser.parse::<ParameterList>().is_ok());
        assert!(parser.is_empty());

        let mut parser = Parser::new(" foo, ").unwrap();
        assert!(parser.parse::<ParameterList>().is_ok());
        assert!(parser.is_empty());

        let mut parser = Parser::new(" foo, bar").unwrap();
        assert!(parser.parse::<ParameterList>().is_ok());
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
    fn test_function_nested() {
        let mut parser = Parser::new("(a) => (b) => c").unwrap();
        let result = parser.parse::<Function>();
        assert!(result.is_ok(), "failed with {:?}", result);
        assert!(parser.is_empty(),);
    }
}
