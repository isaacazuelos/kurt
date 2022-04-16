//! Expressions

use parser::{
    lexer::{Delimiter, Reserved, TokenKind},
    operator::Precedence,
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
    Binary(Binary<'a>),
    Block(Block<'a>),
    Call(Call<'a>),
    Function(Function<'a>),
    Grouping(Grouping<'a>),
    Identifier(Identifier<'a>),
    If(IfElse<'a>),
    List(List<'a>),
    Literal(Literal<'a>),
    Unary(Unary<'a>),
}

impl<'a> Syntax for Expression<'a> {
    const NAME: &'static str = "an expression";

    fn span(&self) -> Span {
        match self {
            Expression::Binary(b) => b.span(),
            Expression::Block(b) => b.span(),
            Expression::Call(c) => c.span(),
            Expression::Function(f) => f.span(),
            Expression::Grouping(g) => g.span(),
            Expression::Identifier(i) => i.span(),
            Expression::If(i) => i.span(),
            Expression::List(l) => l.span(),
            Expression::Literal(e) => e.span(),
            Expression::Unary(u) => u.span(),
        }
    }
}

impl<'a> Parse<'a> for Expression<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Expression<'a>, Error> {
        // TODO: When we have operators, here's where we'll being precedence
        //       climbing, ending with `base`.
        parser.depth_track(|parser| Expression::infix(parser, Precedence::MIN))
    }
}

impl<'a> Expression<'a> {
    /// Parse an expression followed by zero or more infix operators of the
    /// given precedence.
    ///
    /// # Grammar
    ///
    /// infix(max) := prefix
    /// infix(n)   := infix(n+1) (consume_infix(n) (infix(n+1)))*
    ///
    /// What this means is the highest precedence level
    ///
    /// # How this works
    ///
    /// Honestly, I have no idea. I'm not even sure that it does.
    fn infix(
        parser: &mut Parser<'a>,
        precedence: Precedence,
    ) -> Result<Expression<'a>, Error> {
        if precedence == Precedence::MAX {
            return Expression::prefix(parser);
        }

        let mut lhs = Expression::infix(parser, precedence.next())?;
        let mut non_associative_operator_seen = false;

        while let Ok((token, associativity)) = parser.consume_infix(precedence)
        {
            let rhs = match associativity {
                parser::operator::Associativity::Left => {
                    Expression::infix(parser, precedence.next())
                }
                parser::operator::Associativity::Right => {
                    Expression::infix(parser, precedence)
                }
                parser::operator::Associativity::Disallow => {
                    if non_associative_operator_seen {
                        return Err(Error::MultipleNonAssociativeOperators);
                    } else {
                        non_associative_operator_seen = true;
                        Expression::infix(parser, precedence.next())
                    }
                }
            }?;

            lhs = Expression::Binary(Binary::new(token, lhs, rhs))
        }

        Ok(lhs)
    }

    /// Parse a prefix operator expression.
    ///
    /// # Grammar
    ///
    /// Prefix := prefix_operator Prefix
    /// Prefix := base
    fn prefix(parser: &mut Parser<'a>) -> Result<Expression<'a>, Error> {
        parser.depth_track(|parser| {
            if let Ok(token) = parser.consume_prefix() {
                let expression = Expression::prefix(parser)?;
                Ok(Expression::Unary(Unary::new_prefix(token, expression)))
            } else {
                Expression::postfix(parser)
            }
        })
    }

    /// Postfix expressions are primary expressions with
    ///
    /// # Grammar
    ///
    /// Postfix := primary | Call | PostfixOperator
    pub(crate) fn postfix(
        parser: &mut Parser<'a>,
    ) -> Result<Expression<'a>, Error> {
        let mut expression = Expression::primary(parser)?;

        loop {
            expression = match parser.peek() {
                Some(TokenKind::Open(Delimiter::Parenthesis)) => {
                    Call::parse_from(expression, parser)
                        .map(Expression::Call)?
                }
                Some(TokenKind::Operator) => {
                    if let Ok(op) = parser.consume_postfix() {
                        Expression::Unary(Unary::new_postfix(op, expression))
                    } else {
                        break;
                    }
                }

                _ => break,
            }
        }

        Ok(expression)
    }

    /// Primary expressions are expressions which don't themselves have any
    /// suffix parts or operators (i.e. no left or right recursion on expression).
    ///
    /// # Grammar
    ///
    /// primary := Identifier | Block | Function | Literal | List | If
    pub(crate) fn primary(
        parser: &mut Parser<'a>,
    ) -> Result<Expression<'a>, Error> {
        match parser.peek() {
            Some(TokenKind::Reserved(Reserved::If)) => {
                parser.parse().map(Expression::If)
            }

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
        } else if parser.peek_nth(1)
            == Some(TokenKind::Close(Delimiter::Parenthesis))
        {
            let unit = parser.parse()?;
            Ok(Expression::Literal(unit))
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

    use parser::operator::Associativity;

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

    #[test]
    fn parse_unit() {
        let mut parser = Parser::new("()").unwrap();
        let result = parser.parse::<Expression>();
        assert!(
            matches!(&result, Ok(Expression::Literal(l)) if l.kind() == LiteralKind::Unit),
            "expected call but got {:?}",
            result
        );
        assert!(parser.is_empty());
    }

    #[test]
    fn if_else() {
        let mut parser = Parser::new("if true {} else {}").unwrap();
        let result = parser.parse::<Expression>();
        assert!(
            matches!(&result, Ok(Expression::If(_))),
            "expected if-else but got {:?}",
            result
        );
        assert!(parser.is_empty());
    }

    #[test]
    fn if_no_else() {
        let mut parser = Parser::new("if true {}").unwrap();
        let result = parser.parse::<Expression>();
        assert!(result.is_err());
    }

    #[test]
    fn operator_test_assumptions() {
        // make sure some operators are defined the way the following tests
        // think they are.

        let parser = Parser::new("").unwrap();
        assert!(parser.defined_operators().is_prefix("-"));
        assert!(parser.defined_operators().is_postfix("?"));

        let (add_assoc, add_prec) =
            parser.defined_operators().get_infix("+").unwrap();
        let (_mul_assoc, mul_prec) =
            parser.defined_operators().get_infix("*").unwrap();
        let (arrow_assoc, _arrow_prec) =
            parser.defined_operators().get_infix("->").unwrap();
        let (eq_assoc, _eq_prec) =
            parser.defined_operators().get_infix("=").unwrap();

        assert!(add_prec > mul_prec);
        assert_eq!(add_assoc, Associativity::Left);
        assert_eq!(arrow_assoc, Associativity::Right);
        assert_eq!(eq_assoc, Associativity::Disallow);
    }

    #[test]
    fn prefix_operator() {
        let mut parser = Parser::new("-1").unwrap();
        let result = parser.parse::<Expression>();
        assert!(result.is_ok());
    }

    #[test]
    fn postfix_operator() {
        let mut parser = Parser::new("1?").unwrap();
        let result = parser.parse::<Expression>();
        assert!(result.is_ok());
    }

    #[test]
    fn infix_simple() {
        let mut parser = Parser::new("1 + 2").unwrap();
        let result = parser.parse::<Expression>();
        assert!(result.is_ok());
    }

    #[test]
    fn infix_with_unary_operators() {
        let mut parser = Parser::new("-1? + -2?").unwrap();
        let result = parser.parse::<Expression>();
        assert!(
            matches!(result, Ok(Expression::Binary(_))),
            "got {:#?}",
            result
        )
    }

    #[test]
    fn infix_left_associate() {
        let mut parser = Parser::new("1 + 2 + 3").unwrap();
        let result = parser.parse::<Expression>();
        assert!(
            matches!(result, Ok(Expression::Binary(ref b)) if matches!(b.left(), Expression::Binary(_))),
            "got {:#?}",
            result
        )
    }

    #[test]
    fn infix_right_associate() {
        let mut parser = Parser::new("2^3^4").unwrap();
        let result = parser.parse::<Expression>();
        assert!(
            matches!(result, Ok(Expression::Binary(ref b)) if matches!(b.right(), Expression::Binary(_))),
            "got {:#?}",
            result
        )
    }

    #[test]
    fn infix_precedence_higher_right() {
        let mut parser = Parser::new("1 + 2 * 3").unwrap();
        let result = parser.parse::<Expression>();
        assert!(
            matches!(result, Ok(Expression::Binary(ref b)) if b.operator() == "*"),
            "got {:#?}",
            result
        )
    }

    #[test]
    fn infix_precedence_higher_left() {
        let mut parser = Parser::new("1 * 2 + 3").unwrap();
        let result = parser.parse::<Expression>();
        assert!(
            matches!(result, Ok(Expression::Binary(ref b)) if b.operator() == "*"),
            "got {:#?}",
            result
        )
    }

    #[test]
    fn infix_non_associative() {
        let mut parser = Parser::new("1 = 2 = 3").unwrap();
        let result = parser.parse::<Expression>();
        assert!(result.is_err(), "expected error but got {:#?}", result)
    }

    #[test]
    fn infix_non_associative_different_precedences() {
        let mut parser = Parser::new("1 <*> 2 <+> 3").unwrap();

        let add_prec = parser.defined_operators().get_infix("+").unwrap().1;
        let mul_prec = parser.defined_operators().get_infix("*").unwrap().1;

        parser.defined_operators_mut().define_infix(
            "<*>",
            Associativity::Disallow,
            mul_prec,
        );
        parser.defined_operators_mut().define_infix(
            "<+>",
            Associativity::Disallow,
            add_prec,
        );

        let result = parser.parse::<Expression>();
        assert!(
            matches!(result, Ok(Expression::Binary(ref b)) if b.operator() == "<*>"),
            "got {:#?}",
            result
        )
    }
}
