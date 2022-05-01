//! Conditional expressions.

use diagnostic::Span;

use parser::{
    lexer::{Reserved, TokenKind as Kind},
    Parse,
};

use super::*;

/// Conditional expressions, ones with `else` clauses.
///
/// These are different than [`IfOnly`] nodes which don't have `else` blocks,
/// since those can only appear as statements.
///
/// # Grammar
///
/// [`IfElse`] := `if` [`Expression`] [`Block`] `else` [`Block`]
#[derive(Debug)]
pub struct IfElse<'a> {
    if_keyword: Span,
    condition: Box<Expression<'a>>,
    true_block: Block<'a>,
    else_keyword: Span,
    false_block: Block<'a>,
}

impl<'a> IfElse<'a> {
    /// The span of the `if` reserved word.
    pub fn if_span(&self) -> Span {
        self.if_keyword
    }

    /// The condition which is evaluated to branch.
    pub fn condition(&self) -> &Expression<'a> {
        &self.condition
    }

    /// Get a reference to the block run when the condition is true.
    pub fn true_block(&self) -> &Block<'a> {
        &self.true_block
    }

    /// Get a reference to the block run when the condition is not true.
    pub fn false_block(&self) -> &Block<'a> {
        &self.false_block
    }

    /// The span of the `else` reserved word.
    pub fn else_span(&self) -> Span {
        self.else_keyword
    }
}

impl<'a> Syntax for IfElse<'a> {
    fn span(&self) -> Span {
        self.if_keyword + self.false_block.close()
    }
}

impl<'a> Parse<'a> for IfElse<'a> {
    type SyntaxError = SyntaxError;
    fn parse_with(parser: &mut Parser<'a>) -> SyntaxResult<Self> {
        IfOnly::parse_with(parser)?.expand_with_else(parser)
    }
}

/// Conditional expressions which don't have an `else` part.
///
/// See [`IfOnly::expand_with_else`] to try and parse more and turn this into an
/// [`IfElse`].
///
/// # Grammar
///
/// [`IfOnly`] := `if` [`Expression`] [`Block`]
#[derive(Debug)]
pub struct IfOnly<'a> {
    if_keyword: Span,
    condition: Box<Expression<'a>>,
    block: Block<'a>,
}

impl<'a> IfOnly<'a> {
    /// The span of the `if` reserved word.
    pub fn if_span(&self) -> Span {
        self.if_keyword
    }

    /// The condition which is evaluated to branch.
    pub fn condition(&self) -> &Expression<'a> {
        &self.condition
    }

    /// Get a reference to the block run when the condition is true.
    pub fn block(&self) -> &Block<'a> {
        &self.block
    }

    /// Turn an [`IfOnly`] into an [`IfElse`].
    pub fn expand_with_else(
        self,
        parser: &mut Parser<'a>,
    ) -> SyntaxResult<IfElse<'a>> {
        let IfOnly {
            if_keyword,
            condition,
            block,
        } = self;

        let else_keyword = parser
            .consume(Kind::Reserved(Reserved::Else))
            .ok_or(SyntaxError::IfNoElse)?
            .span();

        let false_block = parser.parse()?;

        Ok(IfElse {
            if_keyword,
            condition,
            true_block: block,
            else_keyword,
            false_block,
        })
    }
}

impl<'a> Syntax for IfOnly<'a> {
    fn span(&self) -> Span {
        self.if_keyword + self.block.close()
    }
}

impl<'a> Parse<'a> for IfOnly<'a> {
    type SyntaxError = SyntaxError;

    fn parse_with(parser: &mut Parser<'a>) -> SyntaxResult<Self> {
        let if_keyword = parser
            .consume(Kind::Reserved(Reserved::If))
            .ok_or(SyntaxError::IfNoReserved)?
            .span();

        let condition = Box::new(parser.parse()?);

        let block = parser.parse()?;

        Ok(IfOnly {
            if_keyword,
            condition,
            block,
        })
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_if_only() {
        let mut parser = Parser::new("if true {}").unwrap();
        let syntax = parser.parse::<IfOnly>();
        assert!(syntax.is_ok(), "expected a IfOnly but got {:?}", syntax);
        assert!(parser.is_empty());
    }

    #[test]
    fn test_if_only_when_else() {
        let mut parser = Parser::new("if true {} else {}").unwrap();
        let syntax = parser.parse::<IfOnly>();
        assert!(syntax.is_ok(), "expected a IfElse but got {:?}", syntax);
        assert!(!parser.is_empty());
    }

    #[test]
    fn test_if_else() {
        let mut parser = Parser::new("if true {} else {}").unwrap();
        let syntax = parser.parse::<IfElse>();
        assert!(syntax.is_ok(), "expected a IfElse but got {:?}", syntax);
        assert!(parser.is_empty());
    }

    #[test]
    fn test_else_missing_error() {
        let mut parser = Parser::new("if true {} but no else").unwrap();
        let syntax = parser.parse::<IfElse>();
        assert!(syntax.is_err());
        assert!(!parser.is_empty());
    }
}
