//! Syntax for a whole modules, which is also used as the repl's top-level.

use diagnostic::Span;

use parser::{lexer::TokenKind, Parse};

use super::*;

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
}
