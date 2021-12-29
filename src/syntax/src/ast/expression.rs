//! Expressions

use super::*;

/// This type is a syntax tree enum, like those found in the [`syn`][syn-crate]
/// crate. This means it's an `enum` to dispatch on different types of
/// expressions, each of which is their own actual struct.
///
/// [syn-crate]: https://docs.rs/syn/1.0.84/syn/enum.Expr.html#syntax-tree-enums
pub enum Expression<'a> {
    Literal(Literal<'a>),
}

impl<'a> Syntax for Expression<'a> {
    fn span(&self) -> Span {
        match self {
            Expression::Literal(e) => e.span(),
        }
    }
}

impl<'a> Parse<'a> for Expression<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Expression<'a>, Error> {
        match parser.peek() {
            Some(k) if k.is_literal() => {
                Ok(Expression::Literal(Literal::parse_with(parser)?))
            }

            Some(_) => Err(Error::NotExpression),
            None => Err(Error::EOFExpectingExpression),
        }
    }
}

#[cfg(test)]
mod parser_tests {

    use super::*;

    #[test]
    fn parse_expression_literal() {
        let mut parser = Parser::new("0 ").unwrap();
        let literal = parser.parse::<Expression>();
        assert!(matches!(literal, Ok(Expression::Literal(_))));
        assert!(parser.is_empty());
    }
}
