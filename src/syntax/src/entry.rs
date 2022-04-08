//! Syntax for a module (i.e. an input file).

use diagnostic::Span;

use parser::Parse;

use super::*;

/// A piece of 'top-level' input to some interactive session.
#[derive(Debug)]
pub struct TopLevel<'a> {
    statements: StatementSequence<'a>,
}

impl<'a> TopLevel<'a> {
    /// Get a reference to the top level statements.
    pub fn statements(&self) -> &StatementSequence {
        &self.statements
    }
}

impl<'a> Parse<'a> for TopLevel<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<TopLevel<'a>, Error> {
        StatementSequence::parse_with(parser)
            .map(|statements| TopLevel { statements })
    }
}

impl<'a> Syntax for TopLevel<'a> {
    const NAME: &'static str = "top level";

    fn span(&self) -> Span {
        self.statements.span()
    }
}

/// A module is a piece of a program, a single file of input.
#[derive(Debug)]
pub struct Module<'a> {
    statements: StatementSequence<'a>,
}

impl<'a> Module<'a> {
    /// Get a reference to the top level statements.
    pub fn statements(&self) -> &StatementSequence {
        &self.statements
    }
}

impl<'a> Parse<'a> for Module<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Module<'a>, Error> {
        StatementSequence::parse_with(parser)
            .map(|statements| Module { statements })
    }
}

impl<'a> Syntax for Module<'a> {
    const NAME: &'static str = "module";

    fn span(&self) -> Span {
        self.statements.span()
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
