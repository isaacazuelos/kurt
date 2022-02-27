//! Syntax for a module (i.e. an input file).

use diagnostic::Span;

use crate::lexer::TokenKind;

use super::*;

/// A literal value is something like `123` or `false` which produces a specific
/// value at runtime.
#[derive(Debug)]
pub struct Module<'a> {
    statements: Vec<Statement<'a>>,
    semicolons: Vec<Span>,
}

impl<'a> Module<'a> {
    /// Create a new literal value.
    ///
    /// This will allocate to store the `body`
    pub fn new(
        statements: Vec<Statement<'a>>,
        semicolons: Vec<Span>,
    ) -> Module<'a> {
        Module {
            statements,
            semicolons,
        }
    }

    /// The statements in the module in order.
    pub fn statements(&self) -> &[Statement<'a>] {
        &self.statements
    }

    /// The semicolons that come after the statements.
    ///
    /// Because of the grammar, the length of this is either the same as the
    /// length of [`Module::statements`], or one less if there's no trailing
    /// semicolon.
    pub fn semicolons(&self) -> &[Span] {
        &self.semicolons
    }
}

impl<'a> Syntax for Module<'a> {
    fn span(&self) -> Span {
        if let Some(first) = self.statements.first() {
            first.span() + self.statements.last().unwrap().span()
        } else {
            Span::default()
        }
    }
}

impl<'a> Parse<'a> for Module<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Module<'a>, Error> {
        let mut statements = Vec::new();
        let mut semicolons = Vec::new();

        while !parser.is_empty() {
            statements.push(parser.parse()?);

            if parser.is_empty() {
                break;
            } else if let Some(sep) = parser.consume(TokenKind::Semicolon) {
                semicolons.push(sep.span);
            }
        }

        Ok(Module {
            statements,
            semicolons,
        })
    }
}

#[cfg(test)]
mod parser_tests {

    use super::*;

    #[test]
    fn parse_module_empty() {
        let mut parser = Parser::new("  ").unwrap();
        let literal = parser.parse::<Module>();
        assert!(matches!(literal, Ok(ref m) if m.statements().is_empty()));
        assert!(matches!(literal, Ok(ref m) if m.semicolons().is_empty()));

        assert!(parser.is_empty());
    }

    #[test]
    fn parse_module_empty_semicolon() {
        let mut parser = Parser::new(";").unwrap();
        let literal = parser.parse::<Module>();
        assert!(matches!(literal, Ok(ref m) if m.statements().len() == 1));
        assert!(matches!(literal, Ok(ref m) if m.semicolons().len() == 1));

        assert!(parser.is_empty());
    }

    #[test]
    fn parse_module_no_trailing() {
        let mut parser = Parser::new("0").unwrap();
        let literal = parser.parse::<Module>();
        assert!(matches!(literal, Ok(ref m) if m.statements().len() == 1));
        assert!(matches!(literal, Ok(ref m) if m.semicolons().is_empty()));
        assert!(parser.is_empty());
    }

    #[test]
    fn parse_module_trailing() {
        let mut parser = Parser::new("0;").unwrap();
        let literal = parser.parse::<Module>();
        assert!(matches!(literal, Ok(ref m) if m.statements().len() == 1));
        assert!(matches!(literal, Ok(ref m) if m.semicolons().len() == 1));
        assert!(parser.is_empty());
    }

    #[test]
    fn parse_module_extra_semicolons() {
        let mut parser = Parser::new(";;;").unwrap();
        let literal = parser.parse::<Module>();
        assert!(matches!(literal, Ok(ref m) if m.statements().len() == 3));
        assert!(matches!(literal, Ok(ref m) if m.semicolons().len() == 3));
        assert!(parser.is_empty());
    }
}
