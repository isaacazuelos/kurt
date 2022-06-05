//! Looping constructs.

use diagnostic::Span;

use parser::{
    lexer::{Reserved, TokenKind},
    Parse, Parser,
};

use crate::{Block, Expression, Syntax, SyntaxError, SyntaxResult};

/// A looping construct that never exits unless it hits a `break`. Like Rust's
/// `loop`, used in place of `while true {}`.
///
/// # Grammar
///
/// [`Loop`] := `loop` [`Block`]
#[derive(Debug)]
pub struct Loop<'a> {
    loop_span: Span,
    body: Block<'a>,
}

impl<'a> Loop<'a> {
    pub fn loop_span(&self) -> Span {
        self.loop_span
    }

    pub fn body(&self) -> &Block {
        &self.body
    }
}

impl<'a> Syntax for Loop<'a> {
    fn span(&self) -> Span {
        self.loop_span() + self.body().span()
    }
}

impl<'a> Parse<'a> for Loop<'a> {
    type SyntaxError = SyntaxError;

    fn parse_with(parser: &mut Parser<'a>) -> SyntaxResult<Self> {
        let loop_span = parser
            .consume(TokenKind::Reserved(Reserved::Loop))
            .ok_or_else(|| SyntaxError::LoopNoReserved(parser.next_span()))?
            .span();

        let body = parser.parse()?;

        Ok(Loop { loop_span, body })
    }
}

/// A looping construct that evaluates an expression before each iteration of
/// the loop, and continues with executing the body only if the condition is
/// true.
///
/// # Grammar
///
/// [`While`] := `while` [`Expression`] [`Block`]
#[derive(Debug)]
pub struct While<'a> {
    loop_span: Span,
    condition: Box<Expression<'a>>,
    body: Block<'a>,
}

impl<'a> While<'a> {
    pub fn while_span(&self) -> Span {
        self.loop_span
    }

    pub fn condition(&self) -> &Expression {
        &self.condition
    }

    pub fn body(&self) -> &Block {
        &self.body
    }
}

impl<'a> Syntax for While<'a> {
    fn span(&self) -> Span {
        self.while_span() + self.body().span()
    }
}

impl<'a> Parse<'a> for While<'a> {
    type SyntaxError = SyntaxError;

    fn parse_with(parser: &mut Parser<'a>) -> SyntaxResult<Self> {
        let loop_span = parser
            .consume(TokenKind::Reserved(Reserved::While))
            .ok_or_else(|| SyntaxError::WhileNoReserved(parser.next_span()))?
            .span();

        let condition = Box::new(parser.parse()?);

        let body = parser.parse()?;

        Ok(While {
            loop_span,
            condition,
            body,
        })
    }
}

#[cfg(test)]
mod parser_tests {

    use super::*;

    #[test]
    fn while_loop() {
        let mut parser = Parser::new("while true { 1 }").unwrap();
        assert!(parser.parse::<While>().is_ok());
        assert!(parser.is_empty());
    }

    #[test]
    fn while_loop_no_braces() {
        let mut parser = Parser::new("while (true) return 7;").unwrap();
        assert!(parser.parse::<While>().is_err());
    }

    #[test]
    fn while_weird() {
        let mut parser = Parser::new("while return 7 { ; }").unwrap();
        assert!(parser.parse::<While>().is_ok());
        assert!(parser.is_empty());
    }

    #[test]
    fn while_nested() {
        // this is a bad idea, but it's legal. Like `match match {} {}` in Rust.
        let mut parser = Parser::new("while while false { ; } { ; }").unwrap();
        let result = parser.parse::<While>();
        assert!(result.is_ok(), "failed with {:?}", result);
        assert!(parser.is_empty());
    }

    #[test]
    fn loop_loop() {
        let mut parser = Parser::new("loop {}").unwrap();
        assert!(parser.parse::<Loop>().is_ok());
        assert!(parser.is_empty());
    }

    #[test]
    fn loop_no_block() {
        let mut parser = Parser::new("loop;").unwrap();
        assert!(parser.parse::<Loop>().is_err());
    }
}
