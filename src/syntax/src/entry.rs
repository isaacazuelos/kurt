//! Syntax for a whole modules or the repl top-level.

use diagnostic::Span;

use parser::{lexer::TokenKind, Parse};

use super::*;

/// A piece of 'top-level' input to some interactive session.
///
///
/// # Grammar
///
/// [`TopLevel`] := [`sep_by_trailing`][1]([`Statement`], `;`)
///
/// [1]: Parser::sep_by_trailing
#[derive(Debug)]
pub struct TopLevel<'a> {
    statements: Vec<Statement<'a>>,
    semicolons: Vec<Span>,
}

impl<'a> TopLevel<'a> {
    /// Does this top level input have a trailing semicolon?
    pub fn has_trailing(&self) -> bool {
        !self.semicolons.is_empty()
            && self.semicolons.len() == self.statements.len()
    }
}

impl<'a> Parse<'a> for TopLevel<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<TopLevel<'a>, Error> {
        let (statements, semicolons) =
            parser.sep_by_trailing(TokenKind::Semicolon)?;

        Ok(TopLevel {
            statements,
            semicolons,
        })
    }
}

impl<'a> Syntax for TopLevel<'a> {
    const NAME: &'static str = "top level";

    fn span(&self) -> Span {
        if let Some(first) = self.statements.first() {
            first.span() + self.statements.last().unwrap().span()
        } else {
            Span::default()
        }
    }
}

impl<'a> Sequence for TopLevel<'a> {
    type Element = Statement<'a>;

    const SEPARATOR: TokenKind = TokenKind::Semicolon;

    fn elements(&self) -> &[Self::Element] {
        &self.statements
    }

    fn separators(&self) -> &[Span] {
        &self.semicolons
    }
}

/// A module is a piece of a program, a single file of input.
///
/// # Grammar
///
/// [`Module`] := [`sep_by_trailing`][1]([`Statement`], `;`)
///
/// [1]: Parser::sep_by_trailing
#[derive(Debug)]
pub struct Module<'a> {
    statements: Vec<Statement<'a>>,
    semicolons: Vec<Span>,
}

impl<'a> Module<'a> {
    /// Get a reference to the top level statements.
    pub fn statements(&self) -> &[Statement<'a>] {
        &self.statements
    }

    /// The spans of the semicolons.
    pub fn semicolons(&self) -> &[Span] {
        &self.semicolons
    }

    /// Does this module and in a semicolon?
    pub fn has_trailing(&self) -> bool {
        !self.semicolons.is_empty()
            && self.semicolons.len() == self.statements.len()
    }
}

impl<'a> Parse<'a> for Module<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Module<'a>, Error> {
        let (statements, semicolons) =
            parser.sep_by_trailing(TokenKind::Semicolon)?;

        Ok(Module {
            statements,
            semicolons,
        })
    }
}

impl<'a> Syntax for Module<'a> {
    const NAME: &'static str = "module";

    fn span(&self) -> Span {
        if let Some(first) = self.statements.first() {
            first.span() + self.statements.last().unwrap().span()
        } else {
            Span::default()
        }
    }
}

impl<'a> Sequence for Module<'a> {
    type Element = Statement<'a>;

    const SEPARATOR: TokenKind = TokenKind::Semicolon;

    fn elements(&self) -> &[Self::Element] {
        &self.statements
    }

    fn separators(&self) -> &[Span] {
        &self.semicolons
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn empty_module() {
        let syntax = Module::parse("");
        assert!(syntax.is_ok());
    }

    #[test]
    fn empty_top_level() {
        let syntax = TopLevel::parse(";;;");
        assert!(syntax.is_ok());
    }
}
