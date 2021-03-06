//! Kurt syntax tools
//!
//! The syntax nodes have a 'grammar' listed, but it's not exactly formal.

use diagnostic::Span;

use parser::{
    lexer::{self, TokenKind},
    Parser,
};

pub use parser::{Error, Parse};

mod binding;
mod block;
mod call;
mod conditional;
mod early_exit;
mod entry;
mod error;
mod expression;
mod function;
mod grouping;
mod ident;
mod import;
mod list;
mod literal;
mod loops;
mod operator;
mod statement;
mod subscript;
mod tuple;

pub use self::{
    binding::Binding,
    block::Block,
    call::Call,
    conditional::{IfElse, IfOnly},
    early_exit::{EarlyExit, ExitKind},
    entry::Module,
    error::Error as SyntaxError,
    expression::Expression,
    function::{Function, Parameter},
    grouping::Grouping,
    ident::Identifier,
    import::Import,
    list::List,
    literal::{Kind as LiteralKind, Literal},
    loops::{Loop, While},
    operator::{Binary, Unary},
    statement::Statement,
    subscript::Subscript,
    tuple::Tuple,
};

pub type SyntaxResult<S> = Result<S, parser::Error<SyntaxError>>;

pub trait Syntax: std::fmt::Debug {
    /// The [`Span`] in the original source code that this piece of syntax came
    /// from.
    fn span(&self) -> Span;
}

pub trait Sequence: Syntax {
    type Element;
    const SEPARATOR: TokenKind;

    /// A slice containing the elements of this sequence.
    fn elements(&self) -> &[Self::Element];

    /// The spans of the separators in this sequence.
    fn separators(&self) -> &[Span];

    /// A slice containing the elements of this sequence.
    fn is_empty(&self) -> bool {
        self.elements().is_empty()
    }

    /// Does the block have a trailing semicolon?
    fn has_trailing(&self) -> bool {
        !self.elements().is_empty()
            && self.separators().len() == self.elements().len()
    }
}
