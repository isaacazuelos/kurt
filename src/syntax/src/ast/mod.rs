//! Abstract syntax tree

use diagnostic::Span;

mod binding;
mod entry;
mod expression;
mod ident;
mod literal;
mod statement;

use crate::parser::{Error, Parser};

pub use self::{
    binding::Binding,
    entry::{Module, TopLevel},
    expression::Expression,
    ident::Identifier,
    literal::{Kind as LiteralKind, Literal},
    statement::{Statement, StatementSequence},
};

pub trait Syntax: std::fmt::Debug {
    /// A user-facing name for this piece of syntax.
    ///
    /// These should singular, and include the 'a' or 'an' at the start -- like
    /// 'an expression' or 'a statement'.
    const NAME: &'static str;

    /// The [`Span`] in the original source code that this piece of syntax came
    /// from.
    fn span(&self) -> Span;
}

pub trait Parse<'a>: Sized {
    /// Consume the beginning of the input to parse the expected part of syntax.
    ///
    /// The input may not be empty afterwards, but the parser will have consumed
    /// as much of the input as it can.
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
