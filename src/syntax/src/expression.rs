//! Expressions

use parser::{
    error::OperatorError,
    lexer::{Delimiter, Reserved, TokenKind},
    operator::Precedence,
    Parse,
};

use super::*;

/// Expressions are pieces of the language which evaluate into values.
///
/// This type is a syntax tree enum, like those found in the [`syn`][syn-crate]
/// crate. This means it's an `enum` to dispatch on different types of
/// expressions, each of which is their own actual struct.
///
/// [syn-crate]: https://docs.rs/syn/1.0.84/syn/enum.Expr.html#syntax-tree-enums
///
/// # Grammar
///
/// [`Expression`] := [`infix`][Expression::infix]( [`Precedence::MIN`] )
#[derive(Debug)]
pub enum Expression<'a> {
    Binary(Binary<'a>),
    Block(Block<'a>),
    Call(Call<'a>),
    EarlyExit(EarlyExit<'a>),
    Function(Function<'a>),
    Grouping(Grouping<'a>),
    Identifier(Identifier),
    If(IfElse<'a>),
    List(List<'a>),
    Literal(Literal<'a>),
    Loop(Loop<'a>),
    Subscript(Subscript<'a>),
    Tuple(Tuple<'a>),
    Unary(Unary<'a>),
    While(While<'a>),
}

impl<'a> Syntax for Expression<'a> {
    fn span(&self) -> Span {
        match self {
            Expression::Binary(b) => b.span(),
            Expression::Block(b) => b.span(),
            Expression::Call(c) => c.span(),
            Expression::EarlyExit(e) => e.span(),
            Expression::Function(f) => f.span(),
            Expression::Grouping(g) => g.span(),
            Expression::Identifier(i) => i.span(),
            Expression::If(i) => i.span(),
            Expression::List(l) => l.span(),
            Expression::Literal(e) => e.span(),
            Expression::Loop(l) => l.span(),
            Expression::Subscript(s) => s.span(),
            Expression::Tuple(s) => s.span(),
            Expression::Unary(u) => u.span(),
            Expression::While(w) => w.span(),
        }
    }
}

impl<'a> Parse<'a> for Expression<'a> {
    type SyntaxError = SyntaxError;

    fn parse_with(parser: &mut Parser<'a>) -> SyntaxResult<Expression<'a>> {
        parser.depth_track(|parser| Expression::infix(parser, Precedence::MAX))
    }
}

impl<'a> Expression<'a> {
    /// Parse an expression followed by zero or more infix operators of the
    /// given precedence.
    ///
    /// # Grammar
    ///
    /// - [`infix`][i] (max) := [`prefix`][p]
    /// - [`infix`][i] (n)   := [`infix`][i] (n+1)
    ///                      ([`consume_infix`][c] (n) ( [`infix`][i] (n+1)))*
    ///
    /// What this means is the highest precedence level
    ///
    /// # How this works
    ///
    /// Honestly, I have no idea. I'm not even sure that it does.
    ///
    /// [c]: Parser::consume_infix
    /// [p]: Expression::prefix
    /// [i]: Expression::infix
    pub fn infix(
        parser: &mut Parser<'a>,
        precedence: Precedence,
    ) -> SyntaxResult<Expression<'a>> {
        if precedence == Precedence::MIN {
            return Expression::prefix(parser);
        }

        let mut lhs = Expression::infix(parser, precedence.lower())?;
        let mut non_associative_operator_seen = None;

        while let Ok((token, associativity)) = parser.consume_infix(precedence)
        {
            let rhs = match associativity {
                parser::operator::Associativity::Left => {
                    Expression::infix(parser, precedence.lower())
                }
                parser::operator::Associativity::Right => {
                    Expression::infix(parser, precedence)
                }
                parser::operator::Associativity::Disallow => {
                    if let Some(first) = non_associative_operator_seen {
                        return Err(Error::Operator(
                            OperatorError::MultipleNonAssociative(
                                first,
                                token.span(),
                            ),
                        ));
                    } else {
                        non_associative_operator_seen = Some(token.span());
                        Expression::infix(parser, precedence.lower())
                    }
                }
            }?;

            lhs = Expression::Binary(Binary::new(token, lhs, rhs))
        }

        Ok(lhs)
    }

    /// Prefix operator expressions
    ///
    /// # Grammar
    ///
    /// [`prefix`][0] := [`consume_prefix()`][1] [`prefix`][0]
    ///                | [`postfix`][2]
    ///
    /// [0]: Expression::prefix
    /// [1]: Parser::consume_prefix
    /// [2]: Expression::postfix
    pub fn prefix(parser: &mut Parser<'a>) -> SyntaxResult<Expression<'a>> {
        parser.depth_track(|parser| {
            if let Ok(token) = parser.consume_prefix() {
                let expression = Expression::prefix(parser)?;
                Ok(Expression::Unary(Unary::new_prefix(token, expression)))
            } else {
                Expression::postfix(parser)
            }
        })
    }

    /// Postfix expressions are primary expressions followed by some number of
    /// subscripts, calls, or postfix operators.
    ///
    /// # Grammar
    ///
    /// [`postfix`][0] := [`Call`] | [`Subscript`] | [`primary`][1] [`consume_postfix`][2]
    ///
    /// [0]: Expression::postfix
    /// [2]: Expression::primary
    /// [1]: Parser::consume_postfix
    pub fn postfix(parser: &mut Parser<'a>) -> SyntaxResult<Expression<'a>> {
        let mut expression = Expression::primary(parser)?;

        loop {
            expression = match parser.peek_kind() {
                Some(TokenKind::Open(Delimiter::Parenthesis)) => {
                    Call::parse_from(expression, parser)
                        .map(Expression::Call)?
                }

                Some(TokenKind::Open(Delimiter::Bracket)) => {
                    Subscript::parse_from(expression, parser)
                        .map(Expression::Subscript)?
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
    /// - [`primary`][p] := [`Identifier`] | [`Block`] | [`Function`]
    ///                   | [`Literal`]    | [`List`]  | [`IfOnly`]   
    ///                   | [`IfElse`]     | [`EarlyExit`]
    ///
    /// [p]: Expression::primary
    pub fn primary(parser: &mut Parser<'a>) -> SyntaxResult<Expression<'a>> {
        match parser.peek_kind() {
            Some(TokenKind::Reserved(Reserved::If)) => {
                parser.parse().map(Expression::If)
            }

            Some(TokenKind::Reserved(
                Reserved::Return
                | Reserved::Yield
                | Reserved::Break
                | Reserved::Continue,
            )) => parser.parse().map(Expression::EarlyExit),

            Some(TokenKind::Reserved(Reserved::Loop)) => {
                parser.parse().map(Expression::Loop)
            }

            Some(TokenKind::Reserved(Reserved::While)) => {
                parser.parse().map(Expression::While)
            }

            Some(TokenKind::Identifier) => {
                parser.parse().map(Expression::Identifier)
            }

            Some(TokenKind::Open(Delimiter::Brace)) => {
                // We'll need to do some backtracking here in the future to
                // decide if it's a block or record literal.
                parser.parse().map(Expression::Block)
            }

            // this does each of unit `()`, tuples `(1, 2, 3)`, and function
            // definitions `(a, b) => c`.
            Some(TokenKind::Open(Delimiter::Parenthesis)) => {
                Expression::open_parenthesis(parser)
            }

            Some(TokenKind::Open(Delimiter::Bracket)) => {
                parser.parse().map(Expression::List)
            }

            // this does both keywords literals like `:foo` and tagged
            // collections like `:ok(1)`.
            Some(TokenKind::Colon) => Expression::colon(parser),

            Some(k) if k.is_literal() => {
                parser.parse().map(Expression::Literal)
            }

            Some(_) => Err(Error::Syntax(SyntaxError::ExpressionInvalidStart(
                parser.next_span(),
            ))),

            None => Err(Error::EOF(parser.eof_span())),
        }
    }

    /// Parse an expression which starts with an open paren.
    ///
    /// There are a bunch of syntax that starts with a `(`. Some of these have
    /// arbitrarily-deep prefixes. Instead of parsing and then disambiguating,
    /// we just backtrack.
    ///
    /// - A function's parameter list like `(a, b) => c`
    /// - `()` is unit
    /// - An expression wrapped in parentheses for grouping
    /// - An (untagged) tuple
    ///
    /// If the expression is ambiguous, the first valid interpretation above is
    /// the one used.
    ///
    /// # Grammar
    ///
    /// [`open_parenthesis`][0] := [`Function`] | [`Grouping`]
    ///
    /// [0]: Expression::open_parenthesis
    fn open_parenthesis(
        parser: &mut Parser<'a>,
    ) -> SyntaxResult<Expression<'a>> {
        if let Ok(f) = parser.with_backtracking(Function::parse_with) {
            Ok(Expression::Function(f))
        } else if let Ok(unit) = parser.with_backtracking(Literal::parse_unit) {
            Ok(Expression::Literal(unit))
        } else if let Ok(g) = parser.with_backtracking(Grouping::parse_with) {
            Ok(Expression::Grouping(g))
        } else if let Ok(t) = parser.with_backtracking(Tuple::parse_with) {
            Ok(Expression::Tuple(t))
        } else {
            Err(Error::Syntax(SyntaxError::OpenParenNoParse(
                parser.next_span(),
            )))
        }
    }

    /// Parse an expression which starts with a colon `:`, which could be either
    /// a bare keyword, or it could be part of a tagged collection.
    ///
    /// # Grammar
    ///
    /// [`open_parenthesis`][0] := [`Function`] | [`Grouping`]
    ///
    /// [0]: Expression::open_parenthesis
    fn colon(parser: &mut Parser<'a>) -> SyntaxResult<Expression<'a>> {
        match parser.peek_kind_nth(2) {
            Some(TokenKind::Open(Delimiter::Parenthesis)) => {
                Ok(Expression::Tuple(parser.parse()?))
            }
            // tagged records will go here

            // The literal case handles the error when peek(1) isn't an identifier.
            _ => Ok(Expression::Literal(parser.parse()?)),
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
    fn keyword_not_tuple() {
        let mut parser = Parser::new(" :foo ").unwrap();
        let result = parser.parse::<Expression>();
        assert!(
            matches!(result, Ok(Expression::Literal(_))),
            "expected just a bare keyword literal but got {:?}",
            result
        );
    }

    #[test]
    fn tagged_tuple() {
        let mut parser = Parser::new(" :foo() ").unwrap();
        let result = parser.parse::<Expression>();
        assert!(
            matches!(result, Ok(Expression::Tuple(_))),
            "expected just a bare keyword literal but got {:?}",
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
            matches!(result, Ok(Expression::Binary(ref b)) if b.operator() == "+"),
            "got {:#?}",
            result
        )
    }

    #[test]
    fn infix_precedence_higher_left() {
        let mut parser = Parser::new("1 * 2 + 3").unwrap();
        let result = parser.parse::<Expression>();
        assert!(
            matches!(result, Ok(Expression::Binary(ref b)) if b.operator() == "+"),
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
            // we add _after_ we multiply, so it's the outer node on the tree.
            matches!(result, Ok(Expression::Binary(ref b)) if b.operator() == "<+>"),
            "got {:#?}",
            result
        )
    }

    #[test]
    fn infix_and() {
        let mut parser = Parser::new("true or false and true").unwrap();
        let result = parser.parse::<Expression>();
        assert!(
            parser.defined_operators().get_infix("and").unwrap().1
                < parser.defined_operators().get_infix("or").unwrap().1
        );
        assert!(
            matches!(result, Ok(Expression::Binary(ref b)) if b.operator() == "or")
        );
    }

    #[test]
    fn infix_and_mixed() {
        let mut parser = Parser::new("true or 7 == 3").unwrap();
        let result = parser.parse::<Expression>();
        assert!(
            parser.defined_operators().get_infix("==").unwrap().1
                < parser.defined_operators().get_infix("or").unwrap().1
        );
        assert!(
            matches!(result, Ok(Expression::Binary(ref b)) if b.operator() == "or"),
            "found {:#?}",
            result
        );
    }
}
