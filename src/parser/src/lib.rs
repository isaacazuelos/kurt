//! A Parser-writing tool and a lexer.

pub mod error;
pub mod lexer;
pub mod parser;

pub use crate::{error::Error, parser::Parser};

/// Convert a byte array into a string, but return one of our [`parser::Errors`].
pub fn verify_utf8(input: &[u8]) -> Result<&str, Error> {
    std::str::from_utf8(input)
        .map_err(|e| Error::LexerError(lexer::Error::from(e)))
}

pub trait Parse<'a>: Sized {
    /// Consume the beginning of the input to parse the expected part of syntax.
    ///
    /// The input may not be empty afterwards, but the parser will have consumed
    /// as much of the input as it can.
    fn parse_with(parser: &mut Parser<'a>) -> Result<Self, Error>;

    /// Parse the input to produce the expected syntax type.
    ///
    /// It is an [`Error::UnusedInput`] to not consume the entire input.
    fn parse(input: &'a str) -> Result<Self, Error> {
        let mut parser = Parser::new(input)?;
        let syntax = parser.parse()?;

        if parser.is_empty() {
            Ok(syntax)
        } else {
            Err(Error::UnusedInput)
        }
    }
}
