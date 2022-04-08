//! Lexer errors

use std::{error, fmt};

use crate::lexer::{self, TokenKind as Kind};

/// Lexical errors with all the contextual information needed present it nicely.
#[derive(Debug)]
pub enum Error {
    ParserDepthExceeded,

    NotStartOf(&'static str),
    EOFExpecting(&'static str),

    Unexpected { wanted: &'static str, found: Kind },

    KeywordNoSpace,

    UnusedInput,
    LexerError(lexer::Error),
}

impl Error {
    /// Update the name of the thing we wanted when we encountered the error.
    pub fn set_wanted(self, new: &'static str) -> Error {
        match self {
            Error::NotStartOf(_) => Error::NotStartOf(new),
            Error::EOFExpecting(_) => Error::EOFExpecting(new),
            Error::Unexpected { found, .. } => {
                Error::Unexpected { wanted: new, found }
            }

            other => other,
        }
    }
}

// This [`Display`][fmt::display] implementation doesn't have access to enough
// information to really explain _why_ the error was raised, so these must be
// mostly for presenting to developers working on the language, not for users of
// the language.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            ParserDepthExceeded => {
                write!(f, "parser depth limit exceeded")
            }

            NotStartOf(syntax) => write!(f, "Not the start of {}", syntax),
            EOFExpecting(expected) => {
                write!(f, "Hit end of input when expecting {}", expected)
            }

            Unexpected { wanted, found } => {
                write!(f, "Expected a {} but found a {}", wanted, found.name())
            }

            KeywordNoSpace => write!(
                f,
                "keyword literals must not have a space after the colon"
            ),

            UnusedInput => write!(f, "There was unused input when parsing"),
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
