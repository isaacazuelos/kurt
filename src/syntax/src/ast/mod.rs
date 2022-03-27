//! Abstract syntax tree

use diagnostic::Span;

mod binding;
mod expression;
mod ident;
mod literal;
mod module;
mod statement;

use crate::parser::{Error, Parser};

pub use self::{
    binding::Binding,
    expression::Expression,
    ident::Identifier,
    literal::{Kind as LiteralKind, Literal},
    module::Module,
    statement::Statement,
};

pub trait Syntax: std::fmt::Debug {
    fn span(&self) -> Span;
}

pub trait Parse<'a>: Sized + Syntax {
    /// Consume the beginning of the input to parse the expected part of syntax.
    fn parse_with(parser: &mut Parser<'a>) -> Result<Self, Error>;

    /// Parse the input to produce the expected syntax type.
    ///
    /// It is an [`Error::UnusedInput`] to not consume the entire input.
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
