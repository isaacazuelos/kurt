//! Block expressions, like `{}`

use diagnostic::Span;

use parser::{
    lexer::{Delimiter, TokenKind},
    Parse,
};

use super::*;

/// Block expressions
///
/// A block expression is a brace-delimited and semicolon-separated sequence of
/// statements, like `{ first; second(); }`.
///
/// # Grammar
///
/// [`Block`] := `{` [`sep_by_trailing`][1]([`Statement`], `;`) `}`
///
/// [1]: Parser::sep_by_trailing
#[derive(Debug)]
pub struct Block<'a> {
    open: Span,
    statements: Vec<Statement<'a>>,
    semicolons: Vec<Span>,
    close: Span,
}

impl<'a> Block<'a> {
    /// The span of the block's opening brace.
    pub fn open(&self) -> Span {
        self.open
    }

    /// The span of the block's closing brace.
    pub fn close(&self) -> Span {
        self.close
    }
}

impl<'a> Syntax for Block<'a> {
    fn span(&self) -> Span {
        self.open + self.close
    }
}

impl<'a> Sequence for Block<'a> {
    type Element = Statement<'a>;

    const SEPARATOR: TokenKind = TokenKind::Semicolon;

    fn elements(&self) -> &[Self::Element] {
        &self.statements
    }

    fn separators(&self) -> &[Span] {
        &self.semicolons
    }
}

impl<'a> Parse<'a> for Block<'a> {
    type SyntaxError = SyntaxError;

    fn parse_with(parser: &mut Parser<'a>) -> SyntaxResult<Block<'a>> {
        let open = parser
            .consume(TokenKind::Open(Delimiter::Brace))
            .ok_or_else(|| SyntaxError::BlockNoOpen(parser.next_span()))?
            .span();

        let (statements, semicolons) =
            parser.sep_by_trailing(TokenKind::Semicolon)?;

        let close = parser
            .consume(TokenKind::Close(Delimiter::Brace))
            .ok_or_else(|| SyntaxError::BlockNoClose(open, parser.next_span()))?
            .span();

        Ok(Block {
            open,
            statements,
            semicolons,
            close,
        })
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_empty_block() {
        let mut parser = Parser::new("{}").unwrap();
        let block = parser.parse::<Block>();
        assert!(block.is_ok(), "expected a block but got {:?}", block);
        assert!(block.unwrap().elements().is_empty())
    }

    #[test]
    fn test_block_semicolon_only() {
        let mut parser = Parser::new("{ ; }").unwrap();
        let block = parser.parse::<Block>();
        assert!(
            block.is_ok(),
            "expected a block but got {:?} with state {:#?}",
            block,
            parser
        );
        let block = block.unwrap();
        assert!(block.elements().len() == 1);
        assert!(block.separators().len() == 1);
    }

    #[test]
    fn test_block_multiple_statements() {
        let mut parser = Parser::new("{ 1;2;3; }").unwrap();
        let block = parser.parse::<Block>();
        assert!(
            block.is_ok(),
            "expected a block but got {:?} with state {:#?}",
            block,
            parser
        );
        let block = block.unwrap();
        assert!(block.elements().len() == 3);
        assert!(block.separators().len() == 3);
    }
}
