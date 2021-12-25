//! Lexer errors

use std::{error, fmt};

use diagnostic::Caret;

/// Lexical errors with all the contextual information needed present it nicely.
#[derive(Debug)]
pub enum Error {
    InvalidUnicode(Caret),
    NotStartOfToken(Caret, char),
    Reserved(Caret, char),
    UnexpectedEOF(Caret),
    Unsupported(Caret, &'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
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
