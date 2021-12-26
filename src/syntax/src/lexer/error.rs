//! Lexer errors

use std::{error, fmt};

use diagnostic::Caret;

/// Lexical errors with all the contextual information needed present it nicely.
#[derive(Debug)]
pub enum Error {
    EmptyRadixLiteral(Caret, u32),
    InvalidFloatExponent(Caret),
    InvalidFloatFractional(Caret),
    InvalidUnicode(Caret),
    NotStartOfToken(Caret, char),
    Reserved(Caret, char),
    UnexpectedEOF(Caret),

    // TODO: Finish the cases that would produce this.
    Unsupported(Caret, &'static str),
}

// This [`Display`][fmt::display] implementation doesn't have access to enough
// information to really explain _why_ the error was raised, so these must be
// mostly for presenting to developers working on the language, not _in_ it.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::EmptyRadixLiteral(_, _) => {
                write!(f, "special radix literals can't be empty")
            }
            Error::InvalidFloatExponent(_) => {
                write!(f, "not a valid floating point literal exponent part")
            }
            Error::InvalidFloatFractional(_) => {
                write!(f, "not a valid floating point literal fractional part")
            }
            Error::InvalidUnicode(_) => write!(f, "invalid unicode"),
            Error::NotStartOfToken(_, c) => {
                write!(f, "no token can start with a '{}'", c)
            }
            Error::Reserved(_, c) => {
                write!(f, "the character '{}' is reserved for future use", c)
            }
            Error::UnexpectedEOF(_) => {
                write!(f, "unexpected end of input")
            }

            Error::Unsupported(_, name) => {
                write!(f, "{} tokens are not yet supported", name)
            }
        }
    }
}

impl error::Error for Error {}
