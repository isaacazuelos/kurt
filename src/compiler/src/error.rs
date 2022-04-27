//! Compile time errors

use std::{error, fmt};

use diagnostic::{Diagnostic, Span};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ParseChar(std::char::ParseCharError),
    ParseInt(std::num::ParseIntError),
    ParseFloat(std::num::ParseFloatError),

    MutationNotSupported,
    UndefinedLocal,
    UndefinedPrefix,
    UndefinedInfix,
    UndefinedPostfix,

    CannotBuildWhileCompiling,
    CanOnlyReopenMain,
    CannotReopen,

    TooManyArguments,
    TooManyConstants,
    TooManyOps,
    TooManyParameters,
    TooManyPrototypes,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            ParseChar(e) => write!(f, "cannot parse character literal: {}", e),
            ParseInt(e) => write!(f, "cannot parse integer literal: {}", e),
            ParseFloat(e) => write!(f, "cannot parse float literal: {}", e),

            e => write!(f, "{:?}", e),
        }
    }
}

impl error::Error for Error {}

impl Error {
    fn span(&self) -> Option<Span> {
        None
    }

    fn text(&self) -> String {
        format!("{}", self)
    }
}

impl From<std::char::ParseCharError> for Error {
    fn from(e: std::char::ParseCharError) -> Error {
        Error::ParseChar(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Error {
        Error::ParseInt(e)
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(e: std::num::ParseFloatError) -> Error {
        Error::ParseFloat(e)
    }
}

impl From<Error> for Diagnostic {
    fn from(e: Error) -> Self {
        let mut d = Diagnostic::new(e.text());

        if let Some(span) = e.span() {
            d.set_location(span.start());
        }

        d
    }
}
