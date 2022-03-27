//! Statements

use crate::lexer::TokenKind;

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
            Some(_) => Ok(Statement::Expression(parser.parse()?)),
            None => Err(Error::EOFExpecting("a statement")),
        }
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
    fn parse_expression_with_semicolon() {
        let mut parser = Parser::new("0;").unwrap();
        let literal = parser.parse::<Statement>();
        assert!(matches!(
            literal,
            Ok(Statement::Expression(Expression::Literal(_)))
        ));
        assert!(!parser.is_empty());
    }
}
