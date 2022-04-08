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
}

impl<'a> Syntax for Block<'a> {
    const NAME: &'static str = "a block";

    fn span(&self) -> Span {
        self.open + self.close
    }
}

impl<'a> Parse<'a> for Block<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Block<'a>, Error> {
        let open = match parser.peek() {
            Some(Kind::Open(Delimiter::Brace)) => {
                let token = parser.advance().unwrap();
                token.span()
            }
            Some(found) => {
                return Err(Error::Unexpected {
                    wanted: "an opening brace for a block",
                    found,
                })
            }
            None => {
                return Err(Error::EOFExpecting("an opening brace for a block"))
            }
        };

        let statements =
            if let Some(Kind::Close(Delimiter::Brace)) = parser.peek() {
                StatementSequence::empty()
            } else {
                parser.parse()?
            };

        let close = match parser.peek() {
            Some(Kind::Close(Delimiter::Brace)) => {
                let token = parser.advance().unwrap();
                token.span()
            }
            Some(found) => {
                return Err(Error::Unexpected {
                    wanted: "a closing brace for a block",
                    found,
                })
            }
            None => {
                return Err(Error::EOFExpecting("a closing brace for a block"))
            }
        };

        Ok(Block {
            open,
            statements,
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
