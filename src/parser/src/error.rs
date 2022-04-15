//! Lexer errors

use std::{error, fmt};

use crate::lexer::{self, TokenKind as Kind};

/// Parser errors
///
/// Later we'll add a way to build up more of the context we need for better
/// diagnostics, but this is pretty incomplete for now.
#[derive(Debug)]
pub enum Error {
    ParserDepthExceeded,

    NotStartOf(&'static str),
    EOFExpecting(&'static str),

    Unexpected { wanted: &'static str, found: Kind },

    KeywordNoSpace,

    OperatorNotDefinedAsPrefix,
    OperatorNotDefinedAsPostfix,
    OperatorNotDefinedAsInfix,

    PrefixSpaceAfter,
    PrefixNoSpaceBefore,

    PostfixNoSpaceAfter,
    PostfixSpaceBefore,
    PostfixOperatorAtStartOfInput,

    InfixAtStartOrEnd,
    InfixUnbalancedWhitespace,
    InfixWrongPrecedence,

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

            OperatorNotDefinedAsPrefix => write!(f, "Not a prefix operator"),
            OperatorNotDefinedAsPostfix => write!(f, "Not a postfix operator"),
            OperatorNotDefinedAsInfix => write!(f, "Not an infix operator"),
            PrefixSpaceAfter => write!(f, "Prefix operators cannot have whitespace after them"),
            PrefixNoSpaceBefore => write!(f, "Prefix operators must have whitespace before them"),
            PostfixNoSpaceAfter => write!(f, "Postfix operators must have whitespace after them"),
            PostfixSpaceBefore => write!(f, "Postfix operators cannot have whitespace before them"),
            PostfixOperatorAtStartOfInput => write!(f, "Postfix operators cannot be at the start of the input."),
            InfixAtStartOrEnd => write!(f, "Infix operators must have code on either side"),
            InfixUnbalancedWhitespace => write!(f, "Infix operators must have balanced whitespace"),

            InfixWrongPrecedence  => write!(f, "An infix operator was found, but not with the right precedence."),

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
