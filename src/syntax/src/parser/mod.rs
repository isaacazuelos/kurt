//! A parser.
//!
//! This takes some input

mod error;

use diagnostic::Span;

pub use self::error::Error;

use crate::{
    ast::Parse,
    lexer::{Lexer, Token, TokenKind},
};

/// A parser.
#[derive(Debug)]
pub struct Parser<'a> {
    /// The tokens from our input.
    tokens: Vec<Token<'a>>,

    /// The cursor is the index into the `tokens` which tracks where we've parsed to.
    cursor: usize,

    /// The grammar can be recursive in a few places, we track our 'depth' into
    /// these recursive forms here to prevent stack overflows.
    depth: usize,
}

impl<'a> Parser<'a> {
    /// Create a parser over some input with the default configuration.
    ///
    /// This will immediately return a lexical error if the input isn't
    /// lexically valid.
    ///
    /// # Example
    ///
    /// ```
    /// # use syntax::{ast, parser::Parser};
    /// let mut parser = Parser::new("0").unwrap();
    /// let literal = parser.parse::<ast::Literal>();
    /// assert!(literal.is_ok());
    ///
    /// // Here's an example of it bailing on lexical errors.
    /// let error = Parser::new("'long character literal'");
    /// assert!(error.is_err());
    /// ```
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
        })
    }

    /// Parse the input into syntax.
    ///
    /// # Example
    ///
    /// ```
    /// # use syntax::{ast, parser::Parser};
    /// let mut parser = Parser::new("123;").unwrap();
    /// let module = parser.parse::<ast::Module>();
    /// assert!(module.is_ok());
    ///
    /// let mut parser = Parser::new("not okay :(").unwrap();
    /// let statement = parser.parse::<ast::Module>();
    /// assert!(statement.is_err());
    /// ```
    pub fn parse<T: Parse<'a>>(&mut self) -> Result<T, Error> {
        T::parse_with(self)
    }
}

// Depth tracking
impl<'a> Parser<'a> {
    /// Max expression complexity, in terms of grammar rule recursion.
    ///
    /// Note that the only 2 places where recursion occurs in the grammar will
    /// be for expressions and patterns, both of which are responsible for
    /// preventing the stack from blowing.
    const MAX_DEPTH: usize = 1024;

    /// Increases the depth of the current statement, returning true if the max
    /// depth is hit. This is to prevent parsing from blowing the stack where
    /// the grammar is recursive.
    ///
    /// Don't forget to call [`Parser::decrease_depth`] before all return paths in
    /// a production that calls this to increase.
    pub(crate) fn increase_depth(&mut self) -> bool {
        if self.depth >= Parser::MAX_DEPTH {
            true
        } else {
            self.depth += 1;
            false
        }
    }

    /// Decrease the current depth.
    ///
    /// See [`Parser::increase_depth`] for details.
    pub(crate) fn decrease_depth(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }
}

// Parsing methods
impl<'a> Parser<'a> {
    /// Has the parser hit the end of the input?
    pub(crate) fn is_empty(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    /// Returns the `TokenKind` of the next token, without consuming it.
    pub(crate) fn peek(&self) -> Option<TokenKind> {
        self.peek_nth(0)
    }

    /// Like `Parser::peek` but looking ahead `n` tokens instead of just one.
    ///
    /// Note that this means `peek_n(0)` is like `peek`.
    ///
    /// This returns `None` if there are not `n` more tokens.`
    pub(crate) fn peek_nth(&self, n: usize) -> Option<TokenKind> {
        self.tokens.get(self.cursor + n).map(Token::kind)
    }

    /// The span of the next token. This is sometimes useful when parsing
    /// delimiters.
    pub(crate) fn next_span(&self) -> Option<Span> {
        self.tokens.get(self.cursor).map(Token::span)
    }

    /// Consume the next token, regardless of what it is.
    ///
    /// This returns `None` if there are no more tokens.
    pub(crate) fn advance(&mut self) -> Option<Token<'a>> {
        let token = self.tokens.get(self.cursor).cloned();
        self.cursor += 1;
        token
    }

    /// Advance a token, if it's the specified reserved word.
    ///
    /// This returns `None` if there are no more tokens to consume, or the next
    /// token isn't the expected type.
    pub(crate) fn consume(&mut self, expected: TokenKind) -> Option<Token<'a>> {
        match self.peek() {
            Some(t) if t == expected => self.advance(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod parser_tests {
    use diagnostic::{Caret, Span};

    use super::*;
    use crate::lexer::TokenKind;

    #[test]
    fn is_empty() {
        assert!(Parser::new("").unwrap().is_empty());
        assert!(Parser::new(" ").unwrap().is_empty());
        assert!(!Parser::new("nope").unwrap().is_empty());
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
        assert!(Parser::new("").unwrap().next_span().is_none());
        assert_eq!(
            Parser::new("hi").unwrap().next_span(),
            Some(Span::new(Caret::new(0, 0), Caret::new(0, 2)))
        );
    }

    #[test]
    fn advance() {
        let mut p = Parser::new("hi").unwrap();

        assert!(!p.is_empty());
        assert!(p.advance().is_some());
        assert!(p.is_empty());
        assert!(p.advance().is_none());
    }

    #[test]
    fn consume() {
        let mut p = Parser::new(". a").unwrap();
        assert!(p.consume(TokenKind::Dot).is_some());
        let before = p.cursor;
        assert!(p.consume(TokenKind::Dot).is_none());
        assert_eq!(before, p.cursor);
    }
}
