//! Parser errors

use diagnostic::{Diagnostic, Span};

pub use crate::{
    lexer::Error as LexerError,
    parser::operator_parsing::Error as OperatorError,
};

/// Parser errors
///
/// Later we'll add a way to build up more of the context we need for better
/// diagnostics, but this is pretty incomplete for now.
#[derive(Debug, Clone, Copy)]
pub enum Error<E> {
    Syntax(E),

    EOF(Span),
    ParserDepthExceeded(Span),
    UnconsumedInput(Span),

    Lexer(LexerError),
    Operator(OperatorError),
}

impl<T> From<LexerError> for Error<T> {
    fn from(e: LexerError) -> Self {
        Error::Lexer(e)
    }
}

impl<T> From<OperatorError> for Error<T> {
    fn from(e: OperatorError) -> Self {
        Error::Operator(e)
    }
}

impl<E> From<Error<E>> for Diagnostic
where
    E: Into<Diagnostic>,
{
    #[rustfmt::skip]
    fn from(e: Error<E>) -> Diagnostic {
        match e {
            Error::Syntax(e) => e.into(),

            Error::EOF(span) => {
                Diagnostic::new("input ended unexpectedly")
                    .location(span.start())
                    .highlight(span, "this was the end of the input")
            }
            Error::ParserDepthExceeded(span) => {
                Diagnostic::new("expression too complex to parse")
                    .location(span.start())
                    .highlight(span, "this is where things became too deeply nested")
                    .help("using `let` or functions to break things up might help")
            },
            Error::UnconsumedInput(span) => {
                Diagnostic::new("parsing left unused input")
                    .location(span.start())
                    .highlight(span, "not sure what to do from here on")
                    .info(
                        "this error should usually be replaced by a better \
                        one but wasn't here, please report it"
                    )
            }

            Error::Lexer(l) => l.into(),
            Error::Operator(o) => o.into(),
        }
    }
}
