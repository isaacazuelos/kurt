//! Runtime errors

use std::{error, fmt};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct CastError {
    pub from: &'static str,
    pub to: &'static str,
}

#[derive(Debug)]
pub enum Error {
    NumberTooBig,
    EndOfCode,

    CastError,

    NoMainModule,
    NoMainFunction,

    InvalidArgCount {
        found: u32,
        expected: u32,
    },

    CanOnlyCallClosures,
    SubscriptIndexOutOfRange,

    OperationNotSupported {
        type_name: &'static str,
        op_name: &'static str,
    },

    Cast(CastError),
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            NumberTooBig => write!(f, "number too big"),
            EndOfCode => write!(f, "code ended unexpectedly"),

            CastError => write!(f, "cannot cast value as requested"),

            NoMainModule => write!(f, "no main module is loaded"),
            NoMainFunction => write!(f, "no main function"),

            InvalidArgCount { found, expected } => {
                write!(f, "a function which expected {} arguments was called with {} arguments",
                expected, found
                )
            }
            CanOnlyCallClosures => write!(f, "only closures can be called"),

            SubscriptIndexOutOfRange => {
                write!(f, "subscript index out of range")
            }

            Cast(c) => {
                write!(f, "error casting a {} to {}", c.from, c.to)
            }
            OperationNotSupported { type_name, op_name } => {
                write!(f, "cannot {} with type {}", op_name, type_name)
            }
        }
    }
}

impl From<CastError> for Error {
    fn from(e: CastError) -> Self {
        Error::Cast(e)
    }
}
