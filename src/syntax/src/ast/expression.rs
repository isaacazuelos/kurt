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
    Block(Block<'a>),
    Identifier(Identifier<'a>),
    Literal(Literal<'a>),
}

impl<'a> Syntax for Expression<'a> {
    const NAME: &'static str = "an expression";

    fn span(&self) -> Span {
        match self {
            Expression::Block(b) => b.span(),
            Expression::Literal(e) => e.span(),
            Expression::Identifier(i) => i.span(),
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

            Some(TokenKind::Open(crate::lexer::Delimiter::Brace)) => {
                // We'll need to do some backtracking here in the future to
                // decide if it's a block or record literal.
                parser.parse().map(Expression::Block)
            }

            Some(k) if k.is_literal() => {
                parser.parse().map(Expression::Literal)
            }

            Some(_) => Err(Error::NotStartOf("expression")),

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
