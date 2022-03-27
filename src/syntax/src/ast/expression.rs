//! Expressions

use super::*;

use crate::lexer::TokenKind;

/// This type is a syntax tree enum, like those found in the [`syn`][syn-crate]
/// crate. This means it's an `enum` to dispatch on different types of
/// expressions, each of which is their own actual struct.
///
/// [syn-crate]: https://docs.rs/syn/1.0.84/syn/enum.Expr.html#syntax-tree-enums
#[derive(Debug)]
pub enum Expression<'a> {
    Literal(Literal<'a>),
    Identifier(Identifier<'a>),
}

impl<'a> Syntax for Expression<'a> {
    fn span(&self) -> Span {
        match self {
            Expression::Identifier(i) => i.span(),
            Expression::Literal(e) => e.span(),
        }
    }
}

impl<'a> Parse<'a> for Expression<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Expression<'a>, Error> {
        parser.increase_depth();

        let e = match parser.peek() {
            Some(TokenKind::Identifier) => {
                parser.parse().map(Expression::Identifier)
            }
            Some(k) if k.is_literal() => {
                parser.parse().map(Expression::Literal)
            }

            Some(_) => Err(Error::NotExpression),
            None => Err(Error::EOFExpecting("start of an expression")),
        };
        parser.decrease_depth();

        e
    }
}

#[cfg(test)]
mod parser_tests {

    use super::*;

    #[test]
    fn parse_expression_literal() {
        let mut parser = Parser::new("0 x").unwrap();
        let literal = parser.parse::<Expression>();
        assert!(matches!(literal, Ok(Expression::Literal(_))));

        let ident = parser.parse::<Expression>();
        assert!(matches!(ident, Ok(Expression::Identifier(_))));
        assert!(parser.is_empty());
    }
}
