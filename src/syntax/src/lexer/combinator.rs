//! The functions in here implement tools used in defining production rules in
//! the grammar.
//!
//! If these can fail, they return an [`Option`] rather than an
//! [`Error`][crate::lexer::error::Error]. This is so that the user of these _must_
//! craft the appropriate error rather than passing it up.

use std::str;

use crate::lexer::Lexer;

impl<'a> Lexer<'a> {
    /// Run a rule, but back track if it fails returning [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use syntax::lexer::Lexer;
    /// let mut lexer = Lexer::new("aababb");
    /// let result = lexer.optional(|lex| lex.char('a'));
    /// assert_eq!(result, Some('a'));
    /// assert_eq!(lexer.remaining_input(), "ababb");
    /// ```
    pub fn optional<T, F>(&mut self, rule: F) -> Option<T>
    where
        F: Fn(&mut Lexer<'a>) -> Option<T>,
    {
        let old_location = self.location;
        let old_offset = self.offset;

        let result = rule(self);

        if result.is_none() {
            self.location = old_location;
            self.offset = old_offset;
        }

        result
    }

    /// Get the _n_th character in the input, starting with zero.
    ///
    /// # Notes
    ///
    /// `peek_n(0)` is always the same as `peek()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use syntax::lexer::Lexer;
    /// let lexer = Lexer::new("0123abc");
    /// assert_eq!(lexer.peek_nth(3), Some('3'));
    /// assert_eq!(lexer.remaining_input(), "0123abc");
    /// ```
    pub fn peek_nth(&self, n: usize) -> Option<char> {
        self.remaining_input().chars().nth(n)
    }

    /// Get the next character in the input.
    ///
    /// # Examples
    ///
    /// ```
    /// # use syntax::lexer::Lexer;
    /// let mut lexer = Lexer::new("123abc");
    /// assert_eq!(lexer.peek().unwrap(), '1');
    /// assert_eq!(lexer.remaining_input(), "123abc");
    /// ```
    pub fn peek(&self) -> Option<char> {
        self.peek_nth(0)
    }

    /// Advance the lexer by a single character.
    ///
    /// # Examples
    ///
    /// ```
    /// # use syntax::lexer::Lexer;
    /// let mut lexer = Lexer::new("123abc");
    /// assert_eq!(lexer.advance().unwrap(), '1');
    /// assert_eq!(lexer.remaining_input(), "23abc");
    /// ```
    pub fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;

        self.location.increment(c);
        self.offset += c.len_utf8();

        Some(c)
    }

    /// Consume a specific expected character in the input.
    ///
    /// # Examples
    ///
    /// ```
    /// # use syntax::lexer::Lexer;
    /// let mut lexer = Lexer::new("123abc");
    /// assert_eq!(lexer.char('1').unwrap(), '1');
    /// assert!(lexer.char('b').is_none());
    /// ```
    pub fn char(&mut self, expected: char) -> Option<char> {
        match self.peek() {
            Some(found) if expected == found => Some(self.advance().unwrap()),
            _ => None,
        }
    }

    /// Consume a specific expected string in the input.
    ///
    /// # Examples
    ///
    /// ```
    /// # use syntax::lexer::Lexer;
    /// let mut lexer = Lexer::new("123abc");
    /// let consumed = lexer.str("123").unwrap();
    /// assert_eq!(consumed, "123");
    /// assert_eq!(lexer.remaining_input(), "abc");
    /// ```
    pub fn str<'b>(&mut self, s: &'b str) -> Option<&'b str> {
        for c in s.chars() {
            match self.peek() {
                Some(f) if f == c => {
                    self.advance();
                }
                _ => return None,
            }
        }
        Some(s)
    }

    /// Consume characters in the input while they match a predicate. Might
    /// return an empty string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use syntax::lexer::Lexer;
    /// let mut lexer = Lexer::new("aababbcab");
    /// let consumed = lexer.consume_while(|c| c == 'a' || c == 'b');
    /// assert_eq!(consumed, "aababb");
    /// assert_eq!(lexer.remaining_input(), "cab");
    /// ```
    pub fn consume_while<F>(&mut self, predicate: F) -> &'a str
    where
        F: Fn(char) -> bool,
    {
        let start = self.offset;

        while let Some(c) = self.peek() {
            if predicate(c) {
                self.advance();
            } else {
                break;
            }
        }

        &self.input[start..self.offset]
    }

    /// Consume the next character of input, if it's in the string `cs`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use syntax::lexer::Lexer;
    /// let mut lexer = Lexer::new("abba");
    /// let consumed = lexer.one_of("ab").unwrap();
    /// assert_eq!(consumed, 'a');
    /// assert_eq!(lexer.remaining_input(), "bba");
    /// ```
    pub fn one_of(&mut self, cs: &'static str) -> Option<char> {
        let c = self.peek()?;

        for candidate in cs.chars() {
            if c == candidate {
                return self.char(c);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Just a helper rule used by other tests
    fn rule_a(lex: &mut Lexer) -> Option<char> {
        lex.char('a')
    }

    #[test]
    fn optional_empty() {
        let mut lexer = Lexer::new("babb");
        let consumed = lexer.optional(rule_a);
        assert_eq!(consumed, None);
        assert_eq!(lexer.remaining_input(), "babb");
    }

    #[test]
    fn optional_some() {
        let mut lexer = Lexer::new("ababb");
        let consumed = lexer.optional(rule_a);
        assert_eq!(consumed, Some('a'));
        assert_eq!(lexer.remaining_input(), "babb");
    }

    #[test]
    fn peek() {
        let lex = Lexer::new("ab");
        let result = lex.peek();
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result, 'a');
        assert_eq!(lex.remaining_input(), "ab");
    }

    #[test]
    fn peek_nth() {
        let lex = Lexer::new("ab");
        assert_eq!(lex.peek_nth(0), Some('a'));
        assert_eq!(lex.peek_nth(1), Some('b'));
        assert_eq!(lex.peek_nth(2), None);
    }

    #[test]
    fn advance_empty() {
        let mut lex = Lexer::new("");
        assert_eq!(lex.advance(), None);
    }

    #[test]
    fn str_empty() {
        let mut lex = Lexer::new("");
        assert_eq!(lex.str("a"), None);
        assert_eq!(lex.str(""), Some(""));
    }

    #[test]
    fn consume_while_fail() {
        fn predicate(c: char) -> bool {
            c == 'a'
        }

        let mut lex = Lexer::new("not a single leading a");
        let result = lex.consume_while(predicate);
        assert_eq!(result, "");
    }
}
