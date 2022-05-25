//! Constants are thing like numbers, strings, and other static non-compound
//! values.

use std::{
    char::ParseCharError,
    fmt::{self, Display, Formatter},
    num::{ParseFloatError, ParseIntError},
};

use common::u48;

use crate::error::Error;

/// A constant value (or part of value in the case of closures) which occurs in
/// some code. Some values like `true` don't need to be turned into constants
/// since they can be produced with opcodes directly.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Constant {
    Character(char),
    // stored with `to_bits` for hash/eq reasons
    Float(u64),
    Keyword(String),
    String(String),
}

impl From<char> for Constant {
    fn from(c: char) -> Constant {
        Constant::Character(c)
    }
}

impl From<f64> for Constant {
    fn from(f: f64) -> Constant {
        Constant::Float(f.to_bits())
    }
}

impl From<String> for Constant {
    fn from(s: String) -> Constant {
        Constant::String(s)
    }
}

impl Constant {
    /// Parse the value out of a character literal.
    ///
    /// The input string is expected to include the `'`s that act as delimiters.
    pub fn parse_char(input: &str) -> Result<char, ParseCharError> {
        match input {
            r"'\n'" => Ok('\n'),
            r"'\r'" => Ok('\r'),
            r"'\t'" => Ok('\t'),
            r"'\\'" => Ok('\\'),
            r"'\''" => Ok('\''),
            r#"'\"'"# => Ok('"'),
            _ => {
                // should be ruled out by the lexer.
                debug_assert!(!input.is_empty());
                debug_assert_eq!(input.chars().next(), Some('\''));
                debug_assert_eq!(input.chars().last(), Some('\''));
                let body = &input[1..input.len() - 1];
                let c = body.parse()?;
                Ok(c)
            }
        }
    }

    /// Parse a number into an [`u64`].
    ///
    /// Since our numeric literals don't have have signs (i.e. `-4` is a unary
    /// minus operator) we can used an unsigned integer. It's up to the runtime
    /// to complain during loading if a constant cannot be represented.
    ///
    /// This is weird, but it means the runtime can support different precisions
    /// for numbers or have multiple representations for other constants.
    pub fn parse_int(input: &str) -> Result<u48, ParseIntError> {
        let digits: String = input.chars().filter(|c| *c != '_').collect();

        let n = digits.parse()?;
        Ok(n)
    }

    /// Parse a radix literal.
    ///
    /// See the note on [`parse_int`][Constant::parse_int] for why we use
    /// [`u64`] as the return type.
    pub fn parse_radix(input: &str, radix: u32) -> Result<u48, ParseIntError> {
        // slice off the 0 and radix letter.
        let digits: String = input[2..].chars().filter(|c| *c != '_').collect();
        u48::from_str_radix(&digits, radix)
    }

    /// Parse a floating point number into an [`f64`].
    ///
    /// See the note on [`Constant::parse_int`] about negative values.
    pub fn parse_float(input: &str) -> Result<f64, ParseFloatError> {
        let f = input.parse()?;
        Ok(f)
    }

    /// Parse a string literal.
    ///
    /// For now, escape codes aren't implemented and panic.
    pub fn parse_string(input: &str) -> Result<String, Error> {
        let mut buf = String::new();

        for c in input.chars() {
            match c {
                '\\' => unimplemented!(
                    "escape codes in strings not yet implemented."
                ),
                c => buf.push(c),
            }
        }

        Ok(buf)
    }

    /// Parse a keyword literal.
    ///
    /// For now this is just the body text.
    pub fn parse_keyword(input: &str) -> Self {
        Constant::Keyword(input.to_owned())
    }
}

impl Display for Constant {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Constant::Character(c) => write!(f, "char {c}"),
            Constant::Float(n) => write!(f, "float {}", f64::from_bits(*n)),
            Constant::Keyword(s) => write!(f, "keyword :{s}"),
            Constant::String(s) => write!(f, "string {s}"),
        }
    }
}
