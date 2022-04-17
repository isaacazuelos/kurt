//! Parsers.
//!
//! See the [module documentation][crate] for more information on how this all
//! fits together, and how to use it.

pub mod operator_parsing;

use diagnostic::Span;

use crate::{
    error::Error,
    lexer::{Lexer, Token, TokenKind},
    operator::DefinedOperators,
    Parse,
};

/// A Parser wraps breaks input up into tokens and provides ways to work with
/// that sequence of tokens to define a grammar using [`Parse`].
///
/// See the [module documentation][crate] for more information on how this all
/// fits together, and how to use it.
#[derive(Debug)]
pub struct Parser<'a> {
    /// The tokens from our input.
    tokens: Vec<Token<'a>>,

    /// The cursor is the index into the `tokens` which tracks where we've parsed to.
    cursor: usize,

    /// The grammar can be recursive in a few places, we track our 'depth' into
    /// these recursive forms here to prevent stack overflows.
    depth: usize,

    /// The operators we know how to parse.
    operators: DefinedOperators,
}

impl<'a> Parser<'a> {
    /// Create a parser over some input with the default configuration.
    ///
    /// This will immediately return a lexical error if the input isn't
    /// lexically valid.
    pub fn new(input: &'a str) -> Result<Parser<'a>, Error> {
        let tokens = {
            let mut buf = Vec::new();
            let mut lexer = Lexer::new(input);

            while !lexer.is_empty() {
                buf.push(lexer.token()?);
            }

            buf
        };

        Ok(Parser {
            cursor: 0,
            depth: 0,
            tokens,
            operators: DefinedOperators::default(),
        })
    }

    /// Consume input to produce the specified piece of [`Parse`]able syntax.
    ///
    /// # Note
    ///
    /// Generally you'll want to use [`Parse::parse`] instead, as it ensures
    /// that all input is consumed. This method is instead used for _making_
    /// parsers.
    ///
    /// Some productions could be empty, so it's not unusual for calls to to
    /// return successfully but consume nothing.
    pub fn parse<T: Parse<'a>>(&mut self) -> Result<T, Error> {
        T::parse_with(self)
    }

    /// Has the parser consumed all of the input?
    pub fn is_empty(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    /// Consume the next token if it has the [`TokenKind`] we wanted.. If the
    /// next token has the wrong kind, the error mentioned what we were looking
    /// for using `name`.
    ///
    /// See [`Parser::consume_if`] for more complicated matching.
    pub fn consume(
        &mut self,
        wanted: TokenKind,
        name: &'static str,
    ) -> Result<Token<'a>, Error> {
        self.consume_if(|t| t.kind() == wanted, name)
    }

    /// Consume the next token if it's [`TokenKind`] satisfies the predicated
    /// provided. If the next token has the wrong kind, the error mentioned what
    /// we were looking for using `name`.
    ///
    /// If you just want a specific kind, use [`Parser::consume`] instead.
    ///
    /// Ultimately, this is the only method that moves the parser forward over
    /// input.
    pub fn consume_if(
        &mut self,
        predicate: impl Fn(&Token) -> bool,
        name: &'static str,
    ) -> Result<Token<'a>, Error> {
        match self.tokens.get(self.cursor) {
            Some(found) if predicate(found) => {
                let token = self.tokens[self.cursor];
                self.cursor += 1;
                Ok(token)
            }
            Some(found) => Err(Error::Unexpected {
                wanted: name,
                found: found.kind(),
            }),
            None => Err(Error::EOFExpecting(name)),
        }
    }

    /// Returns the `TokenKind` of the next token, without consuming it.
    pub fn peek(&self) -> Option<TokenKind> {
        self.peek_nth(0)
    }

    /// Like `Parser::peek` but looking ahead `n` tokens instead of just one.
    ///
    /// Note that this means `peek_n(0)` is like `peek`.
    ///
    /// This returns `None` if there are not `n` more tokens.`
    pub fn peek_nth(&self, n: usize) -> Option<TokenKind> {
        self.tokens.get(self.cursor + n).map(Token::kind)
    }

    /// The span of the next token. This is sometimes useful when parsing
    /// delimiters.
    pub fn peek_span(&self) -> Option<Span> {
        self.tokens.get(self.cursor).map(Token::span)
    }

    /// A `sep` separated list of some piece of syntax, with support for
    /// optional trailing separators.
    pub fn sep_by_trailing<S>(
        &mut self,
        sep: TokenKind,
    ) -> Result<(Vec<S>, Vec<Span>), Error>
    where
        S: Parse<'a>,
    {
        let mut elements = Vec::new();
        let mut separators = Vec::new();

        while !self.is_empty() {
            let before = self.cursor;
            match self.parse() {
                Ok(element) => elements.push(element),
                Err(Error::ParserDepthExceeded) => {
                    return Err(Error::ParserDepthExceeded)
                }
                Err(e) => {
                    // If the parser for S consumed some tokens before breaking,
                    // we need to pass that error along -- it means we had a
                    // thing that looked like an S that failed part-way. IF we
                    // need to backtrack properly later, we'll need to be
                    // careful here.
                    if self.cursor != before {
                        return Err(e);
                    }
                }
            }

            match self.peek() {
                // If we see a separator, save it and continue
                Some(t) if t == sep => {
                    let tok = self.consume(t, sep.name()).unwrap();
                    separators.push(tok.span());
                }

                // end of input is a valid end too
                None => break,

                // If it's not the end of input or a sep, whatever's at
                // the end isn't part of the sequence
                Some(_) => break,
            }
        }

        Ok((elements, separators))
    }
}

// Depth tracking
impl<'a> Parser<'a> {
    /// The maximum 'depth' of the parser.
    ///
    /// This only counts parser activity within
    /// [`depth_track`][Parser::depth_track] blocks towards this limit, not just
    /// general grammar depth.
    pub const MAX_DEPTH: usize = 128;

    /// Increases the depth of the current statement, returning true if the max
    /// depth is hit. This is to prevent parsing from blowing the stack where
    /// the grammar is recursive.
    ///
    /// In our grammar, the only two places I forsee this happening is with
    /// expressions and patterns. All statements only contain other statements
    /// within block expressions.
    ///
    /// Likewise, for expressions, it's only in 'primary' expressions which we
    /// need to worry about this.
    pub fn depth_track<F, S>(&mut self, inner: F) -> Result<S, Error>
    where
        F: FnOnce(&mut Self) -> Result<S, Error>,
    {
        if self.depth >= Parser::MAX_DEPTH {
            return Err(Error::ParserDepthExceeded);
        } else {
            self.depth += 1;
        }

        let result = inner(self);

        // If this underflows, something else must have mutated self.depth.
        self.depth -= 1;

        result
    }
}

// Backtracking
impl<'a> Parser<'a> {
    /// Attempt to use the parser for `F`, but on error the parser is returned
    /// to the state it was in before failure.
    #[inline(always)]
    pub fn with_backtracking<S, F>(&mut self, inner: F) -> Result<S, Error>
    where
        F: FnOnce(&mut Self) -> Result<S, Error>,
    {
        let old_depth = self.depth;
        let old_cursor = self.cursor;

        match inner(self) {
            Ok(syntax) => Ok(syntax),
            Err(e) => {
                self.depth = old_depth;
                self.cursor = old_cursor;
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod parser_tests {
    use diagnostic::{Caret, Span};

    use super::*;

    #[test]
    fn consume() {
        let mut p = Parser::new("hi").unwrap();

        assert!(!p.is_empty());
        assert!(p.consume(TokenKind::DoubleArrow, "wrong").is_err());
        assert!(p.consume(TokenKind::Identifier, "identifier").is_ok());
        assert!(p.is_empty());
        assert!(p.consume(TokenKind::DoubleArrow, "wrong").is_err());
    }

    #[test]
    fn consume_if() {
        fn pred(token: &Token) -> bool {
            token.kind() == TokenKind::Identifier
        }

        let mut p = Parser::new("hi").unwrap();

        assert!(!p.is_empty());
        assert!(p.consume_if(pred, "test").is_ok());
        assert!(p.is_empty());
    }

    #[test]
    fn peek() {
        assert!(Parser::new("").unwrap().peek().is_none());
        assert!(Parser::new("a").unwrap().peek().is_some());
    }
    #[test]
    fn peek_nth() {
        assert!(Parser::new("").unwrap().peek_nth(0).is_none());
        assert!(Parser::new("a").unwrap().peek_nth(0).is_some());
        assert!(Parser::new("a").unwrap().peek_nth(1).is_none());
    }

    #[test]
    fn next_span() {
        assert!(Parser::new("").unwrap().peek_span().is_none());
        assert_eq!(
            Parser::new("hi").unwrap().peek_span(),
            Some(Span::new(Caret::new(0, 0), Caret::new(0, 2)))
        );
    }

    #[test]
    fn is_empty() {
        assert!(Parser::new("").unwrap().is_empty());
        assert!(Parser::new(" ").unwrap().is_empty());
        assert!(!Parser::new("nope").unwrap().is_empty());

        let mut parser = Parser::new("hi").unwrap();
        assert!(!parser.is_empty());

        let tok = parser.consume(TokenKind::Identifier, "test");
        assert!(tok.is_ok());
        assert!(parser.is_empty());
    }

    #[test]
    fn backtracking() {
        // lets make sure no backtracking does what we expect.
        let mut parser = Parser::new("1 2").unwrap();
        let result1 = parser.consume(TokenKind::Int, "1");
        let result2 = parser.consume(TokenKind::Int, "2");
        let result3 = parser.consume(TokenKind::Int, "3");

        assert!(result1.is_ok() && result2.is_ok() && result3.is_err());
        assert!(parser.is_empty());

        // Okay now we can try backtracking.

        let mut parser = Parser::new("1 2").unwrap();
        let result = parser.with_backtracking(|p| {
            p.consume(TokenKind::Int, "1")?;
            p.consume(TokenKind::Int, "2")?;
            p.consume(TokenKind::Int, "missing 3")
        });

        assert!(result.is_err());
        assert!(!parser.is_empty());
        assert_eq!(parser.peek_span().map(|s| s.start().column()), Some(0));
    }

    // A few things are tested elsewhere since testing makes more sense with a
    // grammar specified. See tests in `/tests/parser_tests.rs` for more.
    //
    // - `depth_track`
    // - `sep_by_trailing`
}