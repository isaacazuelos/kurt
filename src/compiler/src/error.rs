//! Compile time errors

use std::{error, fmt};

use diagnostic::{Diagnostic, Span};

use crate::{Function, Module};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ParseChar(Span),
    ParseInt(Span, std::num::ParseIntError),
    ParseFloat(Span),

    MutationNotSupported(Span),
    UndefinedLocal(Span),
    UndefinedPrefix(Span),
    UndefinedInfix(Span),
    UndefinedPostfix(Span),

    TooManyArguments(Span),
    TooManyConstants(Span),
    TooManyOps(Span),
    TooManyParameters(Span),
    TooManyPrototypes(Span),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match self {
            ParseChar(_) => {
                write!(f, "cannot parse this character")
            }

            ParseInt(_, _) => write!(f, "cannot parse number"),

            ParseFloat(_) => {
                write!(f, "cannot parse floating-point number")
            }

            MutationNotSupported(_) => {
                write!(f, "mutation with `var` isn't implemented yet")
            }
            UndefinedLocal(_) => write!(f, "no value with this name in scope"),

            UndefinedPrefix(_) => {
                write!(f, "this prefix operator is not defined")
            }
            UndefinedInfix(_) => {
                write!(f, "this infix operator is not defined")
            }
            UndefinedPostfix(_) => {
                write!(f, "this postfix operator is not defined")
            }

            TooManyArguments(_) => {
                write!(f, "this function has too many arguments")
            }
            TooManyConstants(_) => {
                write!(f, "there are too many constant values")
            }
            TooManyOps(_) => write!(f, "this module is too long"),
            TooManyParameters(_) => {
                write!(f, "this function has too many parameters")
            }
            TooManyPrototypes(_) => {
                write!(f, "this module has too many functions")
            }
        }
    }
}

impl error::Error for Error {}

impl Error {
    fn span(&self) -> Span {
        match self {
            Error::ParseChar(s) => *s,
            Error::ParseInt(s, _) => *s,
            Error::ParseFloat(s) => *s,
            Error::MutationNotSupported(s) => *s,
            Error::UndefinedLocal(s) => *s,
            Error::UndefinedPrefix(s) => *s,
            Error::UndefinedInfix(s) => *s,
            Error::UndefinedPostfix(s) => *s,
            Error::TooManyArguments(s) => *s,
            Error::TooManyConstants(s) => *s,
            Error::TooManyOps(s) => *s,
            Error::TooManyParameters(s) => *s,
            Error::TooManyPrototypes(s) => *s,
        }
    }
}

impl From<Error> for Diagnostic {
    fn from(e: Error) -> Self {
        let text = format!("{}", e);
        let location = e.span().start();

        let d = Diagnostic::new(text).location(location);

        match e {
            Error::ParseChar(s) => d.highlight(s, "this character"),
            Error::ParseInt(s, e) => Error::parse_int(d, s, e),
            Error::ParseFloat(s) => d.highlight(s, "this number is the issue"),
            Error::MutationNotSupported(s) => Error::no_mutation(s, d),

            Error::UndefinedLocal(s)
            | Error::UndefinedPrefix(s)
            | Error::UndefinedInfix(s)
            | Error::UndefinedPostfix(s) => {
                d.highlight(s, "no value with this name")
            }

            Error::TooManyArguments(s) => Error::too_many_args(s, d),
            Error::TooManyConstants(s) => Error::too_many_const(s, d),
            Error::TooManyOps(s) => Error::too_many_ops(s, d),
            Error::TooManyParameters(s) => Error::too_many_params(s, d),
            Error::TooManyPrototypes(s) => Error::too_many_prototypes(s, d),
        }
    }
}

impl Error {
    fn no_mutation(s: Span, d: Diagnostic) -> Diagnostic {
        d.highlight(s, "this doesn't work")
            .help("using `let` instead would declare an immutable variable")
    }

    fn too_many_ops(s: Span, d: Diagnostic) -> Diagnostic {
        let info_text = format!(
            "modules and functions compile to a sequence of instructions. \
            Each module or function must fit in {} instructions",
            Function::MAX_OPS
        );

        let help_text = "you can avoid these limits by breaking this into \
            multiple modules and using `import` statements";

        d.highlight(s, "here is where the limit was crossed")
            .info(info_text)
            .help(help_text)
    }

    fn too_many_params(s: Span, d: Diagnostic) -> Diagnostic {
        d.highlight(s, "this parameter is the culprit")
            .info(format!(
                "functions can have a maximum of {} parameters",
                Function::MAX_ARGUMENTS
            ))
    }

    fn too_many_args(s: Span, d: Diagnostic) -> Diagnostic {
        d.highlight(s, "this argument is the culprit").info(format!(
            "function calls can have a maximum of {} arguments",
            Function::MAX_PARAMETERS
        ))
    }

    fn too_many_const(s: Span, d: Diagnostic) -> Diagnostic {
        d.highlight(s, "this constant is where the limit is exceeded")
            .info(format!(
                "each module can only have {} unique literal values",
                Module::MAX_CONSTANTS,
            ))
    }

    fn too_many_prototypes(s: Span, d: Diagnostic) -> Diagnostic {
        d.highlight(s, "this function is the culprit").info(format!(
            "each module can only have {} function definitions",
            Module::MAX_FUNCTIONS - 1,
        ))
    }

    fn parse_int(
        d: Diagnostic,
        s: Span,
        e: std::num::ParseIntError,
    ) -> Diagnostic {
        use std::num::IntErrorKind::*;
        match e.kind() {
            PosOverflow => d
                .highlight(s, "this number is too large")
                .info(
                    "our numbers are 48-bits, which means the largest \
                    supported value is 281,474,976,710,655",
                )
                .help(
                    "floating-point numbers can store bigger numbers, but will \
                    lose precision as the number becomes larger"),
            _ => d.highlight(s, "this number is the issue"),
        }
    }
}
