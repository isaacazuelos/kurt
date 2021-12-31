//! Lexer errors

use std::{error, fmt};

use syntax;

#[derive(Debug)]
pub enum Error {
    Syntax(syntax::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            Syntax(e) => write!(f, "syntax error: {}", e),
        }
    }
}

impl error::Error for Error {}

impl From<syntax::Error> for Error {
    fn from(e: syntax::Error) -> Error {
        Error::Syntax(e)
    }
}
