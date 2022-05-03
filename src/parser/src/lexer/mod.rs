//! Lexing - converting input into [`Token`]s.
//!
//! Generally speaking you shouldn't need these directly, and can go straight to
//! using a [`Parser`][crate::parser::Parser] instead.
//!
//! Before we can start the task of parsing, we need to sweep over the input and
//! breaking it apart into meaningful atoms called [`Token`]s.
//!
//! # Notes
//!
//! You may be wondering why Lexer doesn't implement `Iterator`. The short
//! answer is that it did, returning `Option<Result<Token<'i>, Diagnostic>>` and
//! using it was worse than a `while let` loop like in the example on the struct
//! definition.

mod combinator;
mod error;
mod number;
mod rules;
mod string;
mod token;

use diagnostic::{Caret, Span};

pub use crate::lexer::{
    error::Error,
    token::{Comment, Delimiter, Kind as TokenKind, Reserved, Token},
};

/// A [`Lexer`] scans over a `&str` which scans over the input character by
/// character and breaks things into component meaningful parts ([`Token`]s).
///
/// # Example
///
/// ```
/// # use parser::lexer::Lexer;
/// let mut lexer = Lexer::new("abc def");
/// while let Ok(token) = lexer.token() {
///     // do something with token
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Lexer<'i> {
    /// The input being consumed, as utf8
    pub(crate) input: &'i str,

    /// The location of the caret, as a line and column
    pub(crate) location: Caret,

    /// The location of the caret, as a byte offset
    pub(crate) offset: usize,
}

impl<'i> Lexer<'i> {
    /// Create a new lexer over some input.
    pub fn new(input: &'i str) -> Self {
        let mut lexer = Lexer {
            input,
            location: Caret::default(),
            offset: 0,
        };

        lexer.whitespace();

        lexer
    }

    /// Has the lexer consumed all of the input?
    ///
    /// # Examples
    ///
    /// ```
    /// # use parser::lexer::Lexer;
    /// let lexer = Lexer::new("");
    /// assert!(lexer.is_empty());
    /// let lexer = Lexer::new("non-empty");
    /// assert!(!lexer.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.offset == self.input.len()
    }

    /// Produce the token (or [`Error`]), advancing the lexer.
    ///
    /// If the lexer is empty this will return [`Error::UnexpectedEOF`] since
    /// this call _expects_ to produce a token.
    ///
    /// # Examples
    ///
    /// ```
    /// # use parser::lexer::Lexer;
    /// let mut lexer = Lexer::new("test");
    /// if let Ok(token) = lexer.token() {
    ///     // Do something with the token.
    /// }
    /// ```
    pub fn token(&mut self) -> Result<Token<'i>, Error> {
        if self.is_empty() {
            return Err(Error::UnexpectedEOF(self.location));
        }

        self.whitespace();

        let start_location = self.location;
        let start_offset = self.offset;

        let kind = self.token_kind()?;

        let span = Span::new(start_location, self.location);
        let body = &self.input[start_offset..self.offset];

        self.whitespace();

        Ok(Token { kind, span, body })
    }

    /// The input fed into the lexer that hasn't been broken into tokens yet.
    ///
    /// # Example
    ///
    /// ```
    /// # use parser::lexer::Lexer;
    /// let mut lexer = Lexer::new("abc def");
    /// let abc = lexer.token();
    /// assert_eq!(lexer.remaining_input(), "def");
    /// ```
    pub fn remaining_input(&self) -> &str {
        &self.input[self.offset..]
    }

    /// The [`Span`] of the next [`char`]. If at the end of the input, the span
    /// from is zero-width at the end.
    pub(crate) fn peek_span(&self) -> Span {
        if let Some(next) = self.peek() {
            let mut end = self.location;
            end.increment(next);
            Span::new(self.location, end)
        } else {
            Span::new(self.location, self.location)
        }
    }
}
