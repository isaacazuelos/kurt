//! The rules of the lexical grammar

use unicode_categories::UnicodeCategories;
use unicode_xid::UnicodeXID;

use crate::lexer::{Comment, Delimiter, Error, Lexer, Reserved, TokenKind};

impl Lexer<'_> {
    /// This is the main entry point into the lexer internals. It dispatches to
    /// smaller handlers for more complicated token types.
    pub(crate) fn token_kind(&mut self) -> Result<TokenKind, Error> {
        let next = self.peek().ok_or(Error::UnexpectedEOF(self.location))?;

        match next {
            '@' => {
                self.advance();
                Ok(TokenKind::At)
            }
            ':' => {
                self.advance();
                Ok(TokenKind::Colon)
            }
            ';' => {
                self.advance();
                Ok(TokenKind::Semicolon)
            }
            ',' => {
                self.advance();
                Ok(TokenKind::Comma)
            }
            '(' => {
                self.advance();
                Ok(TokenKind::Open(Delimiter::Parenthesis))
            }
            ')' => {
                self.advance();
                Ok(TokenKind::Close(Delimiter::Parenthesis))
            }
            '[' => {
                self.advance();
                Ok(TokenKind::Open(Delimiter::Bracket))
            }
            ']' => {
                self.advance();
                Ok(TokenKind::Close(Delimiter::Bracket))
            }
            '{' => {
                self.advance();
                Ok(TokenKind::Open(Delimiter::Brace))
            }
            '}' => {
                self.advance();
                Ok(TokenKind::Close(Delimiter::Brace))
            }
            '.' => Ok(self.dots()),

            // Strings
            '\'' => self.character(),
            '\"' => self.string(),

            // Reserved
            '~' | '`' | '#' | '\\' => Err(Error::Reserved(self.location, next)),

            // Comments
            // The single case is covered by operators below.
            '/' if self.peek_nth(1) == Some('/') => Ok(self.comment()),

            // Numbers and words
            c if c.is_digit(10) => self.number(),
            c if is_identifier_start(c) => Ok(self.word()),
            c if is_operator(c) => self.operator(),

            c => Err(Error::NotStartOfToken(self.location, c)),
        }
    }

    /// Whitespace is any string of input which is made up of a sequence of
    /// whitespace characters. It's discarded, which is why this doesn't returns
    /// anything.
    ///
    /// ```text
    /// Whitespace := (Unicode's `White_Space`)*
    /// ```
    pub(crate) fn whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    /// Any number of `.` characters in a row.
    ///
    /// Note that sequences longer than three are an _operator_.
    ///
    /// ```tet
    /// Dots => `.`*
    /// ```
    fn dots(&mut self) -> TokenKind {
        let dots = self.consume_while(|c| c == '.');
        match dots.len() {
            0 => unreachable!("Lexer::dots should only be called after a '.'"),
            1 => TokenKind::Dot,
            2 => TokenKind::Range,
            3 => TokenKind::Spread,
            _ => TokenKind::Operator,
        }
    }

    /// Operators are sequences of symbol- or punctuation-like things.
    ///
    /// Unicode classes are: Pc, Pd, Pe, Pf, Pi, Po, Ps, Sc, Sk, Sm, So.
    /// This also unpacks the special-cased arrow kinds.
    fn operator(&mut self) -> Result<TokenKind, Error> {
        let body = self.consume_while(is_operator);

        match body {
            "" => unreachable!(
                "Lexer::operator should only be called after an operator start"
            ),
            "=" => Ok(TokenKind::Equals),
            "->" => Ok(TokenKind::Arrow),
            "=>" => Ok(TokenKind::DoubleArrow),
            _ => Ok(TokenKind::Operator),
        }
    }

    /// A word is any reserved word or identifier.
    fn word(&mut self) -> TokenKind {
        let start = self.offset;
        let c = self.advance().unwrap();

        debug_assert!(
            is_identifier_start(c),
            "Lexer::word called on non-identifier-start {}",
            c
        );

        let _ = self.consume_while(is_identifier_continue);
        let word = &self.input[start..self.offset];

        if word == "true" || word == "false" {
            return TokenKind::Bool;
        }

        match Reserved::try_from_bytes(word) {
            Some(r) => TokenKind::Reserved(r),
            None => TokenKind::Identifier,
        }
    }

    /// A comment, which is text not used by the program that's written for the
    /// benefit of readers. All comments start with `//` and continue until the
    /// end of the line.
    ///
    /// ```text
    /// _comment_ = `//` followed by characters up to `\n` or the end of input.
    /// ```
    ///
    /// Different kinds of comments exist, based on the character after the
    /// initial `//`.
    ///
    /// - `//!` is a header comment
    /// - `///` is a documentation comment
    /// - `//:` is a markup comment
    /// - `//` starts a line comment
    pub fn comment(&mut self) -> TokenKind {
        self.char('/').unwrap();
        self.char('/').unwrap();

        let kind = match self.peek() {
            Some(':') => Comment::Markup,
            Some('/') => Comment::Doc,
            Some('!') => Comment::Header,
            _ => Comment::Line,
        };

        // We want to consume the char which told us the kind of comment.
        if kind != Comment::Line {
            let _ = self.advance();
        }

        self.consume_while(|c| c != '\n');

        TokenKind::Comment(kind)
    }
}

/// Is a character a valid beginning to an identifier, i.e.
/// [`is_xid_start`][UnicodeXID::is_xid_start] or an underscore?
fn is_identifier_start(c: char) -> bool {
    c == '_' || UnicodeXID::is_xid_start(c)
}

/// Is a character valid inside an identifier, i.e.
/// [`is_xid_continue`][UnicodeXID::is_xid_continue]?
fn is_identifier_continue(c: char) -> bool {
    UnicodeXID::is_xid_continue(c)
}

/// Is a character something we'd consider part of an operator?
fn is_operator(c: char) -> bool {
    c != '@'
        && c != ':'
        && c != ','
        && c != ';'
        && c != '.'
        && c != '"'
        && c != '\''
        && c != '_'
        && c != '{'
        && c != '('
        && c != '['
        && c != ']'
        && c != ')'
        && c != '}'
        && (c.is_symbol() || c.is_punctuation())
}
