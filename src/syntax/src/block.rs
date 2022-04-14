//! Block expressions, like {}

use diagnostic::Span;

use parser::{
    lexer::{Delimiter, TokenKind as Kind},
    Parse,
};

use super::*;

/// A block expression is a brace-delimited and semicolon-separated sequence of
/// statements, like `{ first; second(); }`.
#[derive(Debug)]
pub struct Block<'a> {
    open: Span,
    statements: StatementSequence<'a>,
    close: Span,
}

impl<'a> Block<'a> {
    /// Get a reference to the top level statements.
    pub fn statements(&self) -> &StatementSequence {
        &self.statements
    }

    /// The span of the block's closing brace.
    pub fn close(&self) -> Span {
        self.close
    }
}

impl<'a> Syntax for Block<'a> {
    const NAME: &'static str = "a block";

    fn span(&self) -> Span {
        self.open + self.close
    }
}

impl<'a> Parse<'a> for Block<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Block<'a>, Error> {
        let open = parser
            .consume(
                Kind::Open(Delimiter::Brace),
                "an opening brace for a block",
            )?
            .span();

        let statements =
            if let Some(Kind::Close(Delimiter::Brace)) = parser.peek() {
                StatementSequence::empty()
            } else {
                parser.parse()?
            };

        let close = parser
            .consume(
                Kind::Close(Delimiter::Brace),
                "an closing brace for a block",
            )?
            .span();

        Ok(Block {
            open,
            statements,
            close,
        })
    }

    fn parse(input: &'a str) -> Result<Self, Error> {
        let mut parser = Parser::new(input)?;
        let syntax = parser.parse()?;

        if parser.is_empty() {
            Ok(syntax)
        } else {
            Err(Error::UnusedInput)
        }
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
        assert!(block.unwrap().statements().as_slice().is_empty())
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
        assert!(block.statements().as_slice().len() == 1);
        assert!(block.statements().semicolons().len() == 1);
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
        assert!(block.statements().as_slice().len() == 3);
        assert!(block.statements().semicolons().len() == 3);
    }
}
