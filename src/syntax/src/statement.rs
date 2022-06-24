//! Statements

use parser::Parse;

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
///
/// # Grammar
///
/// [`Statement`] := [`Binding`] | [`IfOnly`] | [`Expression`] | nothing
#[derive(Debug)]
pub enum Statement<'a> {
    Binding(Binding<'a>),
    Empty(Span),
    Expression(Expression<'a>),
    If(IfOnly<'a>),
    Import(Import),
}

impl<'a> Syntax for Statement<'a> {
    fn span(&self) -> Span {
        match self {
            Statement::Import(i) => i.span(),
            Statement::Binding(b) => b.span(),
            Statement::Empty(s) => *s,
            Statement::Expression(s) => s.span(),
            Statement::If(i) => i.span(),
        }
    }
}

impl<'a> Parse<'a> for Statement<'a> {
    type SyntaxError = SyntaxError;

    fn parse_with(parser: &mut Parser<'a>) -> SyntaxResult<Statement<'a>> {
        match parser.peek_kind() {
            Some(TokenKind::Semicolon) => {
                Ok(Statement::Empty(parser.next_span()))
            }

            Some(TokenKind::Reserved(
                Reserved::Var | Reserved::Let | Reserved::Pub,
            )) => Ok(Statement::Binding(parser.parse()?)),

            Some(TokenKind::Reserved(Reserved::Import)) => {
                Ok(Statement::Import(parser.parse()?))
            }

            Some(TokenKind::Reserved(Reserved::If)) => {
                let if_only: IfOnly = parser.parse()?;
                if parser.peek_kind()
                    == Some(TokenKind::Reserved(Reserved::Else))
                {
                    let if_else = if_only.expand_with_else(parser)?;

                    Ok(Statement::Expression(Expression::If(if_else)))
                } else {
                    Ok(Statement::If(if_only))
                }
            }

            Some(_) => Ok(Statement::Expression(parser.parse()?)),

            None => Err(Error::EOF(parser.eof_span())),
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
    fn parse_if_only() {
        let mut parser = Parser::new("if true { }").unwrap();
        let syntax = parser.parse::<Statement>();
        assert!(
            matches!(syntax, Ok(Statement::If(_))),
            "expected If statement, but got {:#?}",
            syntax
        );
        assert!(parser.is_empty());
    }

    #[test]
    fn parse_if_else() {
        let mut parser = Parser::new("if true { } else { }").unwrap();
        let syntax = parser.parse::<Statement>();
        assert!(
            matches!(syntax, Ok(Statement::Expression(Expression::If(_)))),
            "expected If expression, but got {:#?}",
            syntax
        );
        assert!(parser.is_empty());
    }

    #[test]
    fn parse_import() {
        let mut parser = Parser::new("import std").unwrap();
        let syntax = parser.parse::<Statement>();
        assert!(
            matches!(syntax, Ok(Statement::Import(_))),
            "expected an Import statement, but got {:#?}",
            syntax
        );
        assert!(parser.is_empty());
    }
}
