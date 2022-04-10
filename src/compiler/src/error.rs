//! Compile time errors

use std::{error, fmt};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Syntax(syntax::Error),
    ParseChar(std::char::ParseCharError),
    ParseInt(std::num::ParseIntError),
    ParseFloat(std::num::ParseFloatError),

    MutationNotSupported,
    UndefinedLocal,

    TooManyArguments,
    TooManyConstants,
    TooManyOps,
    TooManyPrototypes,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            Syntax(e) => write!(f, "{}", e),
            ParseChar(e) => write!(f, "cannot parse character literal: {}", e),
            ParseInt(e) => write!(f, "cannot parse integer literal: {}", e),
            ParseFloat(e) => write!(f, "cannot parse float literal: {}", e),

            e => write!(f, "error: {:?}", e),
        }
    }
}

impl error::Error for Error {}

impl From<syntax::Error> for Error {
    fn from(e: syntax::Error) -> Error {
        Error::Syntax(e)
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
