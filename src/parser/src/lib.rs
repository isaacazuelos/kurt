//! A parser-writing tool and a lexer.
//!
//! [`Parser`] doesn't parse a specific language, but instead provides tools for
//! writing parsers centered around the [`Parse`] trait where each type of
//! syntax node knows how to parse itself. This is geared towards building
//! recursive decent (i.e. top down) parsers for "mostly LL(k)" grammars, but
//! there are escape hatches for the messy edges.
//!
//! It scans the whole input up front with [`Lexer`][crate::lexer::Lexer], and
//! provides arbitrary lookahead with [`peek_nth`][Parser::peek_nth]. If your
//! grammar needs it, you can backtrack with [`Parser::with_backtracking`].
//!
//! Anywhere your grammar is recursive you should call [`Parser::depth_track`]
//! to help prevent the parser from blowing the stack.
//!
//! There are also tools to help with operator parsing in the [`operator`][op]
//! and [`operator_parsing`][opp] modules.
//!
//! [op]: crate::operator
//! [opp]: crate::parser::operator_parsing

pub mod error;
pub mod lexer;
pub mod operator;
pub mod parser;

pub use crate::{error::Error, parser::Parser};

/// Implementing this trait tells a [`Parser`] how to parse your piece of
/// syntax. The idea is to implement this for as many AST nodes as possible to
/// allow the parser to start parsing at different places in the grammar.
pub trait Parse<'a>: Sized {
    type SyntaxError;

    fn parse(input: &'a str) -> Result<Self, Error<Self::SyntaxError>> {
        let mut parser = Parser::new(input)?;
        let syntax = parser.parse::<Self>()?;

        if parser.is_empty() {
            Ok(syntax)
        } else {
            Err(Error::UnconsumedInput(parser.next_span()))
        }
    }

    /// This is the method used to compose pieces of syntax which implement
    /// [`Parse`] into a larger syntax tree.
    ///
    /// Typically, unless we're done parsing, parser will not be empty
    /// afterwards. Implementations must consume as much as possible before
    /// returning an error.
    fn parse_with(
        parser: &mut Parser<'a>,
    ) -> Result<Self, Error<Self::SyntaxError>>;
}
