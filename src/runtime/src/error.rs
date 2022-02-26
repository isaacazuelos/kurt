//! Lexer errors

use std::{error, fmt};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Syntax(syntax::Error),
    Compiler(compiler::Error),
    NumberTooBig,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            Syntax(e) => write!(f, "syntax error: {}", e),
            Compiler(e) => write!(f, "compiler error: {}", e),
            NumberTooBig => write!(f, "number too big"),
        }
    }
}

impl error::Error for Error {}

impl From<syntax::Error> for Error {
    fn from(e: syntax::Error) -> Error {
        Error::Syntax(e)
    }
}

impl From<compiler::Error> for Error {
    fn from(e: compiler::Error) -> Error {
        Error::Compiler(e)
    }
}