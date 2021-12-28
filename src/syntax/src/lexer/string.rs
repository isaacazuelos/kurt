//! String and character literal lexing.
//!
//! Right now this is a super basic string lexing. There's a lot that we can and
//! probably should support later. Right now we support escapes for the
//! following:
//!
//! - `\n` for newline
//! - `\r` for carriage return
//! - `\t` for tab
//! - `\\` for backslash
//! - `\'` for a single quote
//! - `\"` for a double quote
//!
//! As of yet there's no way to do arbitrary unicode code points or bytes.

// TODO: We should add a few extra flavours of string and escape here.
//
//       1. Unicode escape sequences like "\u{40}".
//       2. Multi-line and raw string literals.
//       3. Interpolation, somehow?
//       4. Raw byte sequences for byte literals like "\x{0}".
//       5. String prefixes like r"" or b""?

use crate::lexer::{Error, Kind, Lexer};

impl Lexer<'_> {
    /// A placeholder for string literals.
    pub(crate) fn string(&mut self) -> Result<Kind, Error> {
        let start = self.location;
        self.char('"').expect("Lexer::string expected a `\"`.");

        loop {
            match self.peek() {
                None => return Err(Error::UnexpectedEOF(self.location)),
                Some('\\') => {
                    self.advance();
                    self.escape_sequence()?
                }
                Some('"') => break,
                Some(_) => {
                    self.advance();
                }
            }
        }

        self.char('"').ok_or_else(|| Error::UnclosedString(start))?;

        Ok(Kind::String)
    }

    /// A placeholder for character literals.
    pub(crate) fn character(&mut self) -> Result<Kind, Error> {
        let start = self.location;
        self.char('\'').expect("Lexer::character expected a `'`.");

        match self.advance() {
            None => return Err(Error::UnexpectedEOF(self.location)),
            Some('\\') => self.escape_sequence()?,
            Some(_) => {}
        };

        self.char('\'')
            .ok_or_else(|| Error::UnclosedCharacter(start))?;
        Ok(Kind::Char)
    }

    fn escape_sequence(&mut self) -> Result<(), Error> {
        match self.peek() {
            None => Err(Error::UnexpectedEOF(self.location)),
            Some('n' | 'r' | 't' | '\\' | '\'' | '\"') => {
                self.advance();
                Ok(())
            }
            Some(c) => Err(Error::InvalidEscape(self.location, c)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character() {
        let mut lexer = Lexer::new("'a'");
        let token = lexer.token().unwrap();
        assert_eq!(token.kind(), Kind::Char);
        assert_eq!(token.body(), "'a'");
    }

    #[test]
    fn character_escape() {
        let mut lexer = Lexer::new("'\\n'");
        let token = lexer.token().unwrap();
        assert_eq!(token.kind(), Kind::Char);
        assert_eq!(token.body(), "'\\n'");
    }

    #[test]
    fn character_invalid_escape() {
        let mut lexer = Lexer::new("'\\s'");
        assert!(lexer.token().is_err());
    }

    #[test]
    fn character_unescaped_double_quote() {
        let mut lexer = Lexer::new("'\"'");
        assert_eq!(lexer.token().unwrap().kind(), Kind::Char);
    }

    #[test]
    fn string() {
        let mut lexer = Lexer::new(r#" "test" "#);
        let token = lexer.token().unwrap();
        assert_eq!(token.kind(), Kind::String);
        assert_eq!(token.body(), r#""test""#);
    }

    #[test]
    fn string_escape() {
        let mut lexer = Lexer::new(r#" "test '\"' " "#);
        let token = lexer.token().unwrap();
        assert_eq!(token.kind(), Kind::String);
        assert_eq!(token.body(), r#""test '\"' ""#);
    }

    #[test]
    fn string_invalid_escape() {
        let mut lexer = Lexer::new(r#" "test \x{0}" "#);
        assert!(lexer.token().is_err());
    }

    #[test]
    fn string_unescaped_single_quote() {
        let mut lexer = Lexer::new(r#" "'" "#);
        assert_eq!(lexer.token().unwrap().kind(), Kind::String);
    }
}
