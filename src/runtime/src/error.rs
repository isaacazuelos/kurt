//! Lexer errors

use std::{error, fmt};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Syntax(syntax::Error),
    Compiler(compiler::Error),
    Format(fmt::Error),

    NumberTooBig,
    EndOfCode,

    ConstantIndexOutOfRange,
    LocalIndexOutOfRange,
    OpIndexOutOfRange,
    PrototypeIndexOutOfRange,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            Syntax(e) => write!(f, "syntax error: {e}"),
            Compiler(e) => write!(f, "compiler error: {e}"),
            Format(e) => write!(f, "formatting error: {e}"),

            NumberTooBig => write!(f, "number too big"),
            EndOfCode => write!(f, "code ended unexpectedly"),

            ConstantIndexOutOfRange => {
                write!(f, "a constant index was out of range")
            }
            LocalIndexOutOfRange => write!(f, "local is out of range"),
            OpIndexOutOfRange => write!(f, "op code index is out of range"),
            PrototypeIndexOutOfRange => {
                write!(f, "function prototype index is out of range")
            }
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

impl From<fmt::Error> for Error {
    fn from(e: fmt::Error) -> Error {
        Error::Format(e)
    }
}
