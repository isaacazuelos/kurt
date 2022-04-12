//! Expressions

use parser::{
    lexer::{Delimiter, TokenKind},
    Parse,
};

use super::*;

/// This type is a syntax tree enum, like those found in the [`syn`][syn-crate]
/// crate. This means it's an `enum` to dispatch on different types of
/// expressions, each of which is their own actual struct.
///
/// [syn-crate]: https://docs.rs/syn/1.0.84/syn/enum.Expr.html#syntax-tree-enums
#[derive(Debug)]
pub enum Expression<'a> {
    Block(Block<'a>),
    Call(Call<'a>),
    Function(Function<'a>),
    Grouping(Grouping<'a>),
    Identifier(Identifier<'a>),
    List(List<'a>),
    Literal(Literal<'a>),
}

impl<'a> Syntax for Expression<'a> {
    const NAME: &'static str = "an expression";

    fn span(&self) -> Span {
        match self {
            Expression::Block(b) => b.span(),
            Expression::Call(c) => c.span(),
            Expression::Function(f) => f.span(),
            Expression::Grouping(g) => g.span(),
            Expression::Identifier(i) => i.span(),
            Expression::List(l) => l.span(),
            Expression::Literal(e) => e.span(),
        }
    }
}

impl<'a> Parse<'a> for Expression<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Expression<'a>, Error> {
        // TODO: When we have operators, here's where we'll being precedence
        //       climbing, ending with `base`.
        Expression::base(parser)
    }
}

impl<'a> Expression<'a> {
    /// Primary expressions are expressions which don't themselves have any
    /// suffix parts or operators (i.e. no left or right recursion on expression).
    ///
    /// # Grammar
    ///
    /// primary := Identifier | Block | Function | Literal | List
    pub(crate) fn primary(
        parser: &mut Parser<'a>,
    ) -> Result<Expression<'a>, Error> {
        parser.depth_track(|parser| {
            match parser.peek() {
                Some(TokenKind::Identifier) => {
                    parser.parse().map(Expression::Identifier)
                }

                Some(TokenKind::Open(Delimiter::Brace)) => {
                    // We'll need to do some backtracking here in the future to
                    // decide if it's a block or record literal.
                    parser.parse().map(Expression::Block)
                }

                Some(TokenKind::Open(Delimiter::Parenthesis)) => {
                    Expression::open_parenthesis(parser)
                }
                Some(TokenKind::Open(Delimiter::Bracket)) => {
                    parser.parse().map(Expression::List)
                }

                Some(k) if k.is_literal() || k == TokenKind::Colon => {
                    parser.parse().map(Expression::Literal)
                }

                Some(_) => Err(Error::NotStartOf(Self::NAME)),

                None => Err(Error::EOFExpecting(Self::NAME)),
            }
        })
    }

    /// Base expressions might have suffixes but don't have any operators.
    ///
    /// # Grammar
    ///
    /// base := primary | Call
    pub(crate) fn base(
        parser: &mut Parser<'a>,
    ) -> Result<Expression<'a>, Error> {
        let mut primary = Expression::primary(parser)?;

        while let Some(TokenKind::Open(Delimiter::Parenthesis)) = parser.peek()
        {
            primary =
                Call::parse_from(primary, parser).map(Expression::Call)?;
        }

        Ok(primary)
    }

    /// Parse an expression which starts with an open paren.
    ///
    /// There are a bunch of things that could be here. Some of these have
    /// arbitrarily-deep prefixes, so we need to either have arbitrary lookahead
    /// and do something like LR parsing with a stack until we decide, or
    /// backtracking.
    ///
    ///
    /// - An expression wrapped in parentheses for grouping
    /// - A function's parameter list
    ///
    /// - Tuples (not yet implemented)
    /// - `()` is unit (not yet implemented)
    ///
    /// # Grammar
    ///
    /// open_parenthesis := Function | Grouping
    fn open_parenthesis(
        parser: &mut Parser<'a>,
    ) -> Result<Expression<'a>, Error> {
        if let Ok(f) = parser.with_backtracking(Function::parse_with) {
            Ok(Expression::Function(f))
        } else {
            Grouping::parse_with(parser)
                .map(Expression::Grouping)
                .map_err(|_| {
                    Error::NotStartOf("a function or grouping expressions")
                })
        }
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

    #[test]
    fn parse_open_paren_grouping() {
        let mut parser = Parser::new("(1)").unwrap();
        let result = parser.parse::<Expression>();
        assert!(matches!(result, Ok(Expression::Grouping(_))));
    }

    #[test]
    fn parse_open_paren_function() {
        let mut parser = Parser::new("() => 1").unwrap();
        let result = parser.parse::<Expression>();
        assert!(
            matches!(result, Ok(Expression::Function(_)),),
            "expected function but got {:?}",
            result,
        );
    }

    #[test]
    fn parse_open_paren_error() {
        let mut parser = Parser::new("( nope").unwrap();
        let result = parser.parse::<Expression>();
        assert!(result.is_err(), "succeeded with {:?}", result);
    }

    #[test]
    fn parse_expression_calls() {
        let mut parser = Parser::new("foo(bar)").unwrap();
        let result = parser.parse::<Expression>();
        assert!(
            matches!(result, Ok(Expression::Call(_))),
            "expected call but got {:?}",
            result
        );
    }

    #[test]
    fn parse_expression_calls_multiple() {
        let mut parser = Parser::new("foo(bar)(baz)").unwrap();
        let result = parser.parse::<Expression>();
        assert!(
            matches!(result, Ok(Expression::Call(_))),
            "expected call but got {:?}",
            result
        );
    }

    #[test]
    fn parse_list() {
        let mut parser = Parser::new("[ 1, 2, 3 ]").unwrap();
        let result = parser.parse::<Expression>();
        assert!(result.is_ok(), "expected list but got {:?}", result);
        assert!(parser.is_empty());
    }

    #[test]
    fn parse_nested_lists() {
        let mut parser = Parser::new("[ 1, [2, [3, nil]]]").unwrap();
        let result = parser.parse::<Expression>();
        assert!(result.is_ok(), "expected list but got {:?}", result);
        assert!(parser.is_empty());
    }
}
