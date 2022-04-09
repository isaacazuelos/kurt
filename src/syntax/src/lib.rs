//! Kurt syntax tools.

use diagnostic::Span;

use parser::{lexer, Parser};

pub use parser::{Error, Parse};

mod binding;
mod block;
mod call;
mod entry;
mod expression;
mod function;
mod grouping;
mod ident;
mod literal;
mod statement;

pub use self::{
    binding::Binding,
    block::Block,
    call::Call,
    entry::{Module, TopLevel},
    expression::Expression,
    function::Function,
    grouping::Grouping,
    ident::Identifier,
    literal::{Kind as LiteralKind, Literal},
    statement::{Statement, StatementSequence},
};

/// Convert a byte array into a string, but return one of our [`parser::Errors`].
pub fn verify_utf8(input: &[u8]) -> Result<&str, Error> {
    std::str::from_utf8(input)
        .map_err(|e| parser::Error::LexerError(lexer::Error::from(e)))
}

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
