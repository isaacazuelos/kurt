//! Compile time errors

use std::{error, fmt};

use diagnostic::{Diagnostic, Span};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ParseChar(Span, std::char::ParseCharError),
    ParseInt(Span, std::num::ParseIntError),
    ParseFloat(Span, std::num::ParseFloatError),

    MutationNotSupported(Span),
    UndefinedLocal(Span),
    UndefinedPrefix(Span),
    UndefinedInfix(Span),
    UndefinedPostfix(Span),

    TooManyArguments(Span),
    TooManyConstants(Span),
    TooManyOps(Span),
    TooManyParameters(Span),
    TooManyPrototypes(Span),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match self {
            ParseChar(_, _) => {
                write!(f, "character literal doesn't make sense")
            }

            ParseInt(_, _) => write!(f, "number cannot be used"),

            ParseFloat(_, _) => {
                write!(f, "floating-point number cannot be used")
            }

            MutationNotSupported(_) => {
                write!(f, "mutation isn't implemented yet")
            }
            UndefinedLocal(_) => write!(f, "no value with this name in scope"),

            UndefinedPrefix(_) => {
                write!(f, "this prefix operator is not defined")
            }
            UndefinedInfix(_) => {
                write!(f, "this infix operator is not defined")
            }
            UndefinedPostfix(_) => {
                write!(f, "this postfix operator is not defined")
            }

            TooManyArguments(_) => {
                write!(f, "this function has too many arguments")
            }
            TooManyConstants(_) => {
                write!(f, "there are too many constants in the module")
            }
            TooManyOps(_) => write!(f, "this module is too long"),
            TooManyParameters(_) => {
                write!(f, "this function has too many parameters")
            }
            TooManyPrototypes(_) => {
                write!(f, "this module has too many functions")
            }
        }
    }
}

impl error::Error for Error {}

impl Error {
    fn span(&self) -> Span {
        match self {
            Error::ParseChar(s, _) => *s,
            Error::ParseInt(s, _) => *s,
            Error::ParseFloat(s, _) => *s,
            Error::MutationNotSupported(s) => *s,
            Error::UndefinedLocal(s) => *s,
            Error::UndefinedPrefix(s) => *s,
            Error::UndefinedInfix(s) => *s,
            Error::UndefinedPostfix(s) => *s,
            Error::TooManyArguments(s) => *s,
            Error::TooManyConstants(s) => *s,
            Error::TooManyOps(s) => *s,
            Error::TooManyParameters(s) => *s,
            Error::TooManyPrototypes(s) => *s,
        }
    }

    fn text(&self) -> String {
        format!("{}", self)
    }
}

impl From<Error> for Diagnostic {
    fn from(e: Error) -> Self {
        Diagnostic::new(e.text())
            .location(e.span().start())
            .highlight(e.span(), e.text())
    }
}
