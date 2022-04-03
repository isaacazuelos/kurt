//! Statements

use crate::lexer::{Reserved, TokenKind};

use super::*;

/// This type is a syntax tree enum, like those found in the [`syn`][syn-crate]
/// crate. This means it's a pretty simple `enum` to dispatch on different types
/// of statements.
///
/// [syn-crate]: https://docs.rs/syn/1.0.84/syn/enum.Expr.html#syntax-tree-enums
///
/// Note that the statement never includes the semicolon at the end (if
/// present).
#[derive(Debug)]
pub enum Statement<'a> {
    Empty(Span),
    Expression(Expression<'a>),
    Binding(Binding<'a>),
}

impl<'a> Syntax for Statement<'a> {
    const NAME: &'static str = "a statement";

    fn span(&self) -> Span {
        match self {
            Statement::Empty(s) => *s,
            Statement::Expression(s) => s.span(),
            Statement::Binding(b) => b.span(),
        }
    }
}

impl<'a> Parse<'a> for Statement<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Statement<'a>, Error> {
        match parser.peek() {
            Some(TokenKind::Semicolon) => {
                Ok(Statement::Empty(parser.next_span().unwrap()))
            }
            Some(TokenKind::Reserved(Reserved::Var | Reserved::Let)) => {
                Ok(Statement::Binding(parser.parse()?))
            }

            Some(_) => Ok(Statement::Expression(parser.parse()?)),
            None => Err(Error::EOFExpecting("a statement")),
        }
    }
}

/// A sequence of [`Statement`]s, with semicolons between them and optionally a
/// trailing semicolon.
#[derive(Debug)]
pub struct StatementSequence<'a> {
    statements: Vec<Statement<'a>>,
    semicolons: Vec<Span>,
}

impl<'a> Syntax for StatementSequence<'a> {
    const NAME: &'static str = "a module";

    fn span(&self) -> Span {
        if let Some(first) = self.statements.first() {
            first.span() + self.statements.last().unwrap().span()
        } else {
            Span::default()
        }
    }
}

impl<'a> Parse<'a> for StatementSequence<'a> {
    fn parse_with(
        parser: &mut Parser<'a>,
    ) -> Result<StatementSequence<'a>, Error> {
        let mut statements = Vec::new();
        let mut semicolons = Vec::new();

        while !parser.is_empty() {
            statements.push(parser.parse()?);

            match parser.peek() {
                // If we see a semicolon, save it and continue
                Some(t) if t == TokenKind::Semicolon => {
                    let sep = parser.advance().unwrap();
                    semicolons.push(sep.span);
                }

                // end of input is a valid end too
                None => break,

                // If it's not the end of input or a semicolon, whatever's at
                // the end isn't part of the statement sequence
                Some(_) => break,
            }
        }

        Ok(StatementSequence {
            statements,
            semicolons,
        })
    }
}

impl<'a> StatementSequence<'a> {
    /// Create a new module.
    pub fn new(
        statements: Vec<Statement<'a>>,
        semicolons: Vec<Span>,
    ) -> StatementSequence<'a> {
        StatementSequence {
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
    /// semicolon. Multiple semicolons are accompanied by a corresponding
    /// [`Statement::Empty`].
    pub fn semicolons(&self) -> &[Span] {
        &self.semicolons
    }

    /// Does this sequence of statements have a trailing semicolon?
    pub fn has_trailing(&self) -> bool {
        !self.semicolons.is_empty()
            && self.semicolons.len() == self.statements.len()
    }
}

#[cfg(test)]
mod parser_tests {

    use super::*;

    #[test]
    fn parse_expression_literal() {
        let mut parser = Parser::new("0").unwrap();
        let literal = parser.parse::<Statement>();
        assert!(matches!(
            literal,
            Ok(Statement::Expression(Expression::Literal(_)))
        ));
        assert!(parser.is_empty());
    }

    #[test]
    fn parse_empty() {
        let mut parser = Parser::new(";").unwrap();
        let literal = parser.parse::<Statement>();
        assert!(matches!(literal, Ok(Statement::Empty(_))));
        assert!(!parser.is_empty());
    }

    #[test]
    fn parse_binding() {
        let mut parser = Parser::new("let x = 1;").unwrap();
        let literal = parser.parse::<Statement>();
        assert!(matches!(literal, Ok(Statement::Binding(_))));
        assert!(!parser.is_empty());
    }

    #[test]
    fn parse_expression_with_semicolon() {
        let mut parser = Parser::new("0;").unwrap();
        let literal = parser.parse::<Statement>();
        assert!(matches!(
            literal,
            Ok(Statement::Expression(Expression::Literal(_)))
        ));
        assert!(!parser.is_empty());
    }

    #[test]
    fn parse_statements_empty() {
        let mut parser = Parser::new("  ").unwrap();
        let literal = parser.parse::<StatementSequence>();
        assert!(matches!(literal, Ok(ref m) if m.statements().is_empty()));
        assert!(matches!(literal, Ok(ref m) if m.semicolons().is_empty()));

        assert!(parser.is_empty());
    }

    #[test]
    fn parse_statements_empty_semicolon() {
        let mut parser = Parser::new(";").unwrap();
        let literal = parser.parse::<StatementSequence>();
        assert!(matches!(literal, Ok(ref m) if m.statements().len() == 1));
        assert!(matches!(literal, Ok(ref m) if m.semicolons().len() == 1));

        assert!(parser.is_empty());
    }

    #[test]
    fn parse_statements_no_trailing() {
        let mut parser = Parser::new("0").unwrap();
        let literal = parser.parse::<StatementSequence>();
        assert!(matches!(literal, Ok(ref m) if m.statements().len() == 1));
        assert!(matches!(literal, Ok(ref m) if m.semicolons().is_empty()));
        assert!(parser.is_empty());
    }

    #[test]
    fn parse_statements_trailing_semicolon() {
        let mut parser = Parser::new("0;").unwrap();
        let literal = parser.parse::<StatementSequence>();
        assert!(matches!(literal, Ok(ref m) if m.statements().len() == 1));
        assert!(matches!(literal, Ok(ref m) if m.semicolons().len() == 1));
        assert!(parser.is_empty());
    }

    #[test]
    fn parse_statements_extra_semicolons() {
        let mut parser = Parser::new(";;;").unwrap();
        let literal = parser.parse::<StatementSequence>();
        assert!(matches!(literal, Ok(ref m) if m.statements().len() == 3));
        assert!(matches!(literal, Ok(ref m) if m.semicolons().len() == 3));
        assert!(parser.is_empty());
    }

    #[test]
    fn parse_statements_trailing() {
        let mut parser = Parser::new("1 1").unwrap();
        let literal = parser.parse::<StatementSequence>();
        assert!(
            matches!(literal, Ok(ref m) if m.statements().len() == 1),
            "expected 1 statement, but got {:#?}",
            literal
        );
        assert!(!parser.is_empty());
    }
}
