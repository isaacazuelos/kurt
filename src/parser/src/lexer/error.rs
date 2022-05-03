//! Lexer errors

use std::{error, fmt};

use diagnostic::{Caret, Diagnostic, Span};

/// Lexical errors with the contextual information needed present it nicely.
#[derive(Debug, Clone, Copy)]
pub enum Error {
    EmptyRadixLiteral(Span, u32),
    InvalidEscape(Span, char),
    EmptyFloatExponent(Span, char),
    EmptyFloatFractional(Span),
    NotStartOfToken(Span, char),
    Reserved(Span, char),
    UnclosedCharacter(Span, Span),
    UnclosedString(Span, Span),
    UnexpectedEOF(Caret),
}

// This [`Display`][fmt::display] implementation doesn't have access to enough
// information to really explain _why_ the error was raised, so these must be
// mostly for presenting to developers working on the language, not _in_ it.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::EmptyRadixLiteral(_, 2) => {
                write!(f, "binary literals can't be empty")
            }
            Error::EmptyRadixLiteral(_, 8) => {
                write!(f, "octal literals can't be empty")
            }
            Error::EmptyRadixLiteral(_, 16) => {
                write!(f, "hexadecimal literals can't be empty")
            }
            Error::EmptyRadixLiteral(_, _) => {
                write!(f, "special radix literals can't be empty")
            }
            Error::InvalidEscape(_, c) => {
                write!(f, "not a valid escape sequence '{}'", c)
            }
            Error::EmptyFloatExponent(_, _) => {
                write!(f, "a floating point number's exponent part doesn't have any digits")
            }
            Error::EmptyFloatFractional(_) => {
                write!(
                    f,
                    "a floating point number's fractional part is missing"
                )
            }
            Error::NotStartOfToken(_, c) => {
                write!(f, "`{}` is not the start of any valid token", c)
            }
            Error::Reserved(_, c) => {
                write!(f, "the character '{}' is reserved for future use", c)
            }
            Error::UnclosedCharacter(_, _) => {
                write!(f, "a character is missing it's closing `'`")
            }
            Error::UnclosedString(_, _) => {
                write!(f, "a string is missing it's closing `\"`")
            }
            Error::UnexpectedEOF(_) => {
                write!(f, "the input ended unexpectedly")
            }
        }
    }
}

impl error::Error for Error {}

impl From<Error> for Diagnostic {
    fn from(e: Error) -> Self {
        let d = Diagnostic::new(format!("{e}")).location(e.location());

        match e {
            Error::EmptyRadixLiteral(s, n) => Error::empty_radix(e, s, n),

            Error::NotStartOfToken(s, _) => {
                d.highlight(s, "not sure what to make of this")
            }

            Error::InvalidEscape(s, _) => {
                d.highlight(s, "this is not a valid escape sequence").info(
                    "supported escape sequences are:\n\
                    - `\\n` for newlines,\n\
                    - `\\r` for carriage returns,\n\
                    - `\\t` for tabs,\n\
                    - `\\\\` for backslashes,\n\
                    - `\\\'` for single quotes,\n\
                    - `\\\"` for double quotes,",
                )
            }

            Error::Reserved(s, _) => {
                d.highlight(s, "this character doesn't mean anything, yet")
            }

            Error::EmptyFloatExponent(s, e) => d
                .highlight(
                    s,
                    format!("an exponent was expected because of this `{}`", e),
                )
                .help(format!("either remove the `{}` or add an exponent", e)),

            Error::EmptyFloatFractional(s) => d.highlight(
                s,
                "this `.` means this is a floating point number, \
                but the digits after the `.` are missing",
            ),

            Error::UnclosedCharacter(open, close) => d
                .highlight(open, "the character started here")
                .highlight(close, "expected a `'` here"),

            Error::UnclosedString(open, close) => d
                .highlight(open, "the string started here")
                .highlight(close, "should have seen it's closing `\"` by here"),

            // probably not useful to point and say "here" huh?
            Error::UnexpectedEOF(_) => d,
        }
    }
}

impl Error {
    fn location(&self) -> Caret {
        match self {
            Error::EmptyRadixLiteral(s, _) => s.start(),
            Error::InvalidEscape(s, _) => s.start(),
            Error::EmptyFloatExponent(s, _) => s.start(),
            Error::EmptyFloatFractional(s) => s.end(),
            Error::NotStartOfToken(s, _) => s.start(),
            Error::Reserved(s, _) => s.start(),
            Error::UnclosedCharacter(_, s) => s.start(),
            Error::UnclosedString(_, s) => s.start(),
            Error::UnexpectedEOF(c) => *c,
        }
    }

    fn empty_radix(e: Error, s: Span, n: u32) -> Diagnostic {
        let d = Diagnostic::new(format!("{e}"))
            .location(e.location())
            .highlight(s, "this looks like the start of a number");

        match n {
            2 => d.info("only numbers can start with `0`, and binary numbers start with `0b`"),
            8 => d.info("only numbers can start with `0`, and octal (base 8) numbers start with `0o`"),
            16 => d.info("only numbers can start with `0`, and hexadecimal (base 16) numbers start with `0x`"),
            _ => d,
        }
    }
}
