//! Operator syntax nodes.
//!
//! Notable, these do not implement [`Parse`][crate::Parse].
//!
//! [`Unary`] would need to understand primary and base expressions for things
//! like `-a.?b!`, and [`Binary`] would need to know at what precedence. Both
//! are so intertwined with [`Expression`], it doesn't really make sense to
//! parse them outside of that context.

use diagnostic::Span;

use parser::lexer::Token;

use crate::{Expression, Syntax};

/// Unary expressions
///
/// Both prefix and postfix
///
/// # Grammar
///
/// [`Unary`] := [`Expression::prefix`] | [`Expression::postfix`]
#[derive(Debug)]
pub struct Unary<'a> {
    token: Token<'a>,
    is_prefix: bool,
    operand: Box<Expression<'a>>,
}

impl<'a> Unary<'a> {
    pub fn new_prefix(token: Token<'a>, operand: Expression<'a>) -> Unary<'a> {
        Unary {
            token,
            operand: Box::new(operand),
            is_prefix: true,
        }
    }

    pub fn new_postfix(token: Token<'a>, operand: Expression<'a>) -> Unary<'a> {
        Unary {
            token,
            operand: Box::new(operand),
            is_prefix: false,
        }
    }

    /// The body of the operator token.
    pub fn operator(&self) -> &str {
        self.token.body()
    }

    /// The span of the operator token.
    pub fn operator_span(&self) -> Span {
        self.token.span()
    }

    /// Was this unary operator prefix (or postfix)?
    pub fn is_prefix(&self) -> bool {
        self.is_prefix
    }

    /// Get a operand, the expression it's applied to.
    pub fn operand(&self) -> &Expression {
        self.operand.as_ref()
    }
}

impl Syntax for Unary<'_> {
    const NAME: &'static str = "a unary operator";

    fn span(&self) -> Span {
        self.token.span() + self.operand.span()
    }
}

/// Binary expressions
///
/// # Grammar
///
/// [`Binary`] := [`Expression::infix`]
#[derive(Debug)]
pub struct Binary<'a> {
    token: Token<'a>,
    operands: Box<(Expression<'a>, Expression<'a>)>,
}

impl<'a> Binary<'a> {
    pub fn new(
        token: Token<'a>,
        lhs: Expression<'a>,
        rhs: Expression<'a>,
    ) -> Binary<'a> {
        Binary {
            token,
            operands: Box::new((lhs, rhs)),
        }
    }

    /// The body of the operator token.
    pub fn operator(&self) -> &str {
        self.token.body()
    }

    /// The span of the operator token.
    pub fn operator_span(&self) -> Span {
        self.token.span()
    }

    /// Get a left hand side of the binary expression.
    pub fn left(&self) -> &Expression {
        &self.operands.0
    }

    /// Get a right hand side of the binary expression.
    pub fn right(&self) -> &Expression {
        &self.operands.1
    }
}

impl Syntax for Binary<'_> {
    const NAME: &'static str = "a binary operator";

    fn span(&self) -> Span {
        self.left().span() + self.right().span()
    }
}
