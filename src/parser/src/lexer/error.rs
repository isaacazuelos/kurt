//! Lexer errors

use std::{error, fmt};

use diagnostic::{Caret, Diagnostic, Span};

/// Lexical errors with the contextual information needed present it nicely.
#[derive(Debug, Clone, Copy)]
pub enum Error {
    EmptyRadixLiteral(Span, u32),
    InvalidEscape(Caret, char),
    InvalidFloatExponent(Caret),
    InvalidFloatFractional(Caret),
    InvalidUnicodeEscape(Caret),
    NotStartOfToken(Caret, char),
    Reserved(Caret, char),
    UnclosedCharacter(Caret),
    UnclosedString(Caret),
    UnexpectedEOF(Caret),
}

// This [`Display`][fmt::display] implementation doesn't have access to enough
// information to really explain _why_ the error was raised, so these must be
// mostly for presenting to developers working on the language, not _in_ it.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::EmptyRadixLiteral(_, 2) => {
                write!(f, "binary literals can't be empty")
            }
            Error::EmptyRadixLiteral(_, 8) => {
                write!(f, "octal literals can't be empty")
            }
            Error::EmptyRadixLiteral(_, 16) => {
                write!(f, "hexadecimal literals can't be empty")
            }
            Error::EmptyRadixLiteral(_, _) => {
                write!(f, "special radix literals can't be empty")
            }
            Error::InvalidEscape(_, c) => {
                write!(f, "not a valid escape sequence '{}'", c)
            }
            Error::InvalidFloatExponent(_) => {
                write!(f, "not a valid floating point literal exponent part")
            }
            Error::InvalidFloatFractional(_) => {
                write!(f, "not a valid floating point literal fractional part")
            }
            Error::InvalidUnicodeEscape(_) => write!(f, "invalid unicode"),
            Error::NotStartOfToken(_, c) => {
                write!(f, "no token can start with a '{}'", c)
            }
            Error::Reserved(_, c) => {
                write!(f, "the character '{}' is reserved for future use", c)
            }
            Error::UnclosedCharacter(_) => {
                write!(f, "character literal is missing closing single quote")
            }
            Error::UnclosedString(_) => {
                write!(f, "string literal is missing closing double quote")
            }
            Error::UnexpectedEOF(_) => {
                write!(f, "unexpected end of input")
            }
        }
    }
}

impl error::Error for Error {}

impl From<Error> for Diagnostic {
    fn from(e: Error) -> Self {
        let d = Diagnostic::new(format!("{e}")).location(e.location());

        match e {
            Error::EmptyRadixLiteral(s, n) => Error::empty_radix(e, s, n),
            Error::NotStartOfToken(_, _) => d,
            Error::InvalidEscape(_, _) => d,
            Error::Reserved(_, _) => d,

            Error::InvalidFloatExponent(_) => d,
            Error::InvalidFloatFractional(_) => d,
            Error::InvalidUnicodeEscape(_) => d,
            Error::UnclosedCharacter(_) => d,
            Error::UnclosedString(_) => d,
            Error::UnexpectedEOF(_) => d,
        }
    }
}

impl Error {
    fn location(&self) -> Caret {
        match self {
            Error::EmptyRadixLiteral(s, _) => s.start(),
            Error::InvalidEscape(c, _) => *c,
            Error::InvalidFloatExponent(c) => *c,
            Error::InvalidFloatFractional(c) => *c,
            Error::InvalidUnicodeEscape(c) => *c,
            Error::NotStartOfToken(c, _) => *c,
            Error::Reserved(c, _) => *c,
            Error::UnclosedCharacter(c) => *c,
            Error::UnclosedString(c) => *c,
            Error::UnexpectedEOF(c) => *c,
        }
    }

    fn empty_radix(e: Error, s: Span, n: u32) -> Diagnostic {
        let d = Diagnostic::new(format!("{e}"))
            .location(e.location())
            .highlight(s, "this looks like the start of a number");

        match n {
            2 => d.info("only numbers can start with `0`, and binary numbers start with `0b`"),
            8 => d.info("only numbers can start with `0`, and octal (base 8) numbers start with `0o`"),
            16 => d.info("only numbers can start with `0`, and hexadecimal (base 16) numbers start with `0x`"),
            _ => d,
        }
    }
}
