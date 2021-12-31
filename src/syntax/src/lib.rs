//! Kurt syntax tools.

pub mod ast;
pub mod lexer;
pub mod parser;

pub use crate::{
    ast::{Module, Parse, Syntax},
    parser::Error,
};

/// Convert a byte array into a string, but return an
pub fn verify_utf8(input: &[u8]) -> Result<&str, parser::Error> {
    std::str::from_utf8(input)
        .map_err(|e| parser::Error::LexerError(lexer::Error::from(e)))
}
