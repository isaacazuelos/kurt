//! Early exit expressions, `yield`, `return`, `continue` and `break`,
//! expressions which are a reserved word followed by an optional expression.

use diagnostic::Span;

use parser::{
    lexer::{Reserved, TokenKind},
    Parse,
};

use crate::{Expression, Syntax, SyntaxError};

#[derive(Debug, Clone, Copy)]
pub enum ExitKind {
    Return,
    Yield,
    Continue,
    Break,
}

#[derive(Debug)]
pub struct EarlyExit<'a> {
    kind: ExitKind,
    span: Span,
    expression: Option<Box<Expression<'a>>>,
}

impl<'a> EarlyExit<'a> {
    pub fn kind(&self) -> ExitKind {
        self.kind
    }

    pub fn expression(&self) -> Option<&Expression> {
        self.expression.as_deref()
    }
}

impl<'a> Syntax for EarlyExit<'a> {
    fn span(&self) -> Span {
        if let Some(expression) = &self.expression {
            self.span + expression.span()
        } else {
            self.span
        }
    }
}

impl<'a> Parse<'a> for EarlyExit<'a> {
    type SyntaxError = SyntaxError;

    fn parse_with(
        parser: &mut parser::Parser<'a>,
    ) -> Result<Self, parser::Error<Self::SyntaxError>> {
        let token = parser
            .consume_if(|t| {
                matches!(
                    t.kind(),
                    TokenKind::Reserved(Reserved::Return)
                        | TokenKind::Reserved(Reserved::Yield)
                        | TokenKind::Reserved(Reserved::Continue)
                        | TokenKind::Reserved(Reserved::Break)
                )
            })
            .ok_or_else(|| {
                SyntaxError::EarlyExitNoReservedWord(parser.next_span())
            })?;

        let kind = match token.kind() {
            TokenKind::Reserved(Reserved::Return) => ExitKind::Return,
            TokenKind::Reserved(Reserved::Yield) => ExitKind::Yield,
            TokenKind::Reserved(Reserved::Continue) => ExitKind::Continue,
            TokenKind::Reserved(Reserved::Break) => ExitKind::Break,
            _ => unreachable!(), // see match in consume_if above
        };

        let expression = match parser.with_backtracking(|p| p.parse()) {
            Ok(e) => Some(Box::new(e)),
            Err(_) => None,
        };

        Ok(EarlyExit {
            kind,
            span: token.span(),
            expression,
        })
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    use parser::Parser;

    #[test]
    fn return_no_expression() {
        let mut parser = Parser::new("return + 1").unwrap();
        let result = parser.parse::<EarlyExit>();
        assert!(result.is_ok(), "failed with {:?}", result);
    }

    #[test]
    fn return_with_expression() {
        let mut parser = Parser::new("return 2 + 1").unwrap();
        let result = parser.parse::<EarlyExit>();
        assert!(result.is_ok(), "failed with {:?}", result);
        assert!(
            result.unwrap().expression().is_some(),
            "expression wasn't picked up"
        );
    }
}
