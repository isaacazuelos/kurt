//! Lexer errors

use std::{error, fmt};

use crate::lexer::{self, TokenKind as Kind};

/// Lexical errors with all the contextual information needed present it nicely.
#[derive(Debug)]
pub enum Error {
    NotExpression,
    EOFExpecting(&'static str),

    Unexpected { wanted: Kind, found: Kind },

    UnusedInput,
    LexerError(lexer::Error),
}

// This [`Display`][fmt::display] implementation doesn't have access to enough
// information to really explain _why_ the error was raised, so these must be
// mostly for presenting to developers working on the language, not for users of
// the language.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            NotExpression => write!(f, "Not the start of an expression"),
            EOFExpecting(expected) => {
                write!(f, "Hit end of input when expecting {}", expected)
            }

            Unexpected { wanted, found } => {
                write!(
                    f,
                    "Expected a {} but found a {}",
                    wanted.name(),
                    found.name()
                )
            }

            UnusedInput => write!(f, "there was unused input when parsing"),
            LexerError(e) => write!(f, "{}", e),
        }
    }
}

impl error::Error for Error {}

impl From<lexer::Error> for Error {
    fn from(e: lexer::Error) -> Error {
        Error::LexerError(e)
    }
}
