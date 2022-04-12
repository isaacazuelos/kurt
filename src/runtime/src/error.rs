//! Lexer errors

use std::{error, fmt};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NumberTooBig,
    EndOfCode,

    CastError,

    NoMainModule,
    NoMainFunction,

    InvalidArgCount,
    StackIndexBelowZero,
    CanOnlyCallClosures,
    CannotReturnFromTop,

    ConstantIndexOutOfRange,
    LocalIndexOutOfRange,
    ModuleIndexOutOfRange,
    OpIndexOutOfRange,
    PrototypeIndexOutOfRange,
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

            InvalidArgCount => {
                write!(f, "function call has wrong number of arguments")
            }
            StackIndexBelowZero => write!(f, "stack indexed below zero"),
            CanOnlyCallClosures => write!(f, "only closures can be called"),
            CannotReturnFromTop => write!(f, "cannot return from top"),

            ConstantIndexOutOfRange => {
                write!(f, "a constant index was out of range")
            }
            LocalIndexOutOfRange => write!(f, "local is out of range"),
            ModuleIndexOutOfRange => write!(f, "module out of range"),
            OpIndexOutOfRange => write!(f, "op code index is out of range"),
            PrototypeIndexOutOfRange => {
                write!(f, "function prototype index is out of range")
            }
        }
    }
}
