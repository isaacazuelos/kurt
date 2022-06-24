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

    VarNotSupported(Span),
    RecNotFunction(Span, Span),
    EarlyExitKindNotSupported(Span),
    NotALegalAssignmentTarget(Span),
    ContinueWithValue(Span),
    ShadowExport(Span, Span),
    PubNotTopLevel(Span),
    ImportNotTopLevel(Span),

    JumpTooFar(Span),

    UndefinedLocal(Span),
    UndefinedPrefix(Span),
    UndefinedInfix(Span),
    UndefinedPostfix(Span),

    TooManyArguments(Span),
    TooManyConstants(Span),
    TooManyOps(Span),
    TooManyParameters(Span),
    TooManyFunctions(Span),
    TooManyLocals(Span),
    TooManyExports(Span),
    TooManyImports(Span),
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

            VarNotSupported(_) => {
                write!(f, "bindings are created with `let`, not `var`")
            }
            RecNotFunction(_, _) => {
                write!(f, "recursive bindings only supported on functions")
            }
            EarlyExitKindNotSupported(_) => {
                write!(f, "cannot yet compile this expression")
            }
            NotALegalAssignmentTarget(_) => {
                write!(f, "cannot assign to this")
            }
            ContinueWithValue(_) => {
                write!(f, "a `continue` cannot take a value")
            }
            ShadowExport(_, _) => write!(f, "cannot shadow exported value"),
            JumpTooFar(_) => {
                write!(f, "this code needs to jump too far")
            }
            ImportNotTopLevel(_) => write!(f, "`import` must be at top-level"),
            PubNotTopLevel(_) => write!(
                f,
                "bindings which are `pub` must be at the top-level scope"
            ),

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
            TooManyFunctions(_) => {
                write!(f, "this module has too many functions")
            }
            TooManyLocals(_) => {
                write!(f, "this function has too many local bindings")
            }
            TooManyExports(_) => {
                write!(f, "this module has to many exported bindings")
            }
            TooManyImports(_) => {
                write!(f, "this module has to many imported modules")
            }
        }
    }
}

impl error::Error for Error {}

impl Error {
    fn span(&self) -> Span {
        *match self {
            Error::ParseChar(s) => s,
            Error::ParseInt(s, _) => s,
            Error::ParseFloat(s) => s,
            Error::VarNotSupported(s) => s,
            Error::RecNotFunction(_, s) => s,
            Error::EarlyExitKindNotSupported(s) => s,
            Error::NotALegalAssignmentTarget(s) => s,
            Error::ContinueWithValue(s) => s,
            Error::ShadowExport(s, _) => s,
            Error::JumpTooFar(s) => s,
            Error::PubNotTopLevel(s) => s,
            Error::ImportNotTopLevel(s) => s,
            Error::UndefinedLocal(s) => s,
            Error::UndefinedPrefix(s) => s,
            Error::UndefinedInfix(s) => s,
            Error::UndefinedPostfix(s) => s,
            Error::TooManyArguments(s) => s,
            Error::TooManyConstants(s) => s,
            Error::TooManyOps(s) => s,
            Error::TooManyParameters(s) => s,
            Error::TooManyFunctions(s) => s,
            Error::TooManyLocals(s) => s,
            Error::TooManyExports(s) => s,
            Error::TooManyImports(s) => s,
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

            Error::VarNotSupported(s) => Error::var(s, d),
            Error::RecNotFunction(rec, s) => Error::rec_not_function(rec, s, d),
            Error::EarlyExitKindNotSupported(s) => {
                Error::early_kind_not_supported(s, d)
            }
            Error::NotALegalAssignmentTarget(s) => {
                Error::not_assignment_target(s, d)
            }
            Error::ContinueWithValue(s) => Error::continue_with_value(s, d),
            Error::ShadowExport(s, p) => Error::shadow_export(s, p, d),
            Error::JumpTooFar(s) => Error::jump_too_far(s, d),
            Error::PubNotTopLevel(s) => Error::pub_not_top_level(s, d),
            Error::ImportNotTopLevel(s) => Error::import_not_top_level(s, d),

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
            Error::TooManyFunctions(s) => Error::too_many_functions(s, d),
            Error::TooManyLocals(s) => Error::too_many_locals(s, d),
            Error::TooManyExports(s) => Error::too_many_exports(s, d),
            Error::TooManyImports(s) => Error::too_many_imports(s, d),
        }
    }
}

impl Error {
    fn var(s: Span, d: Diagnostic) -> Diagnostic {
        d.highlight(s, "use `let` here")
            .help("use `let` here instead of `var`")
    }

    fn rec_not_function(rec: Span, s: Span, d: Diagnostic) -> Diagnostic {
        d.highlight(rec, "this can only be used if defining a function")
            .highlight(s, "this is not a function")
            .help("this will be supported (hopefully) soon")
    }

    fn early_kind_not_supported(span: Span, d: Diagnostic) -> Diagnostic {
        d.highlight(span, "this isn't implemented yet")
            .help("it's in the works, but only `return` works right now")
    }

    fn not_assignment_target(span: Span, d: Diagnostic) -> Diagnostic {
        d.highlight(span, "can't assign to this")
            .help("you can only assign to variables, or subscripts")
    }

    fn continue_with_value(s: Span, d: Diagnostic) -> Diagnostic {
        d.highlight(s, "this is the code that's a problem")
            .help("breaking this up with functions might help")
    }

    fn jump_too_far(s: Span, d: Diagnostic) -> Diagnostic {
        d.highlight(s, "this value isn't allowed").info(
            "Unlike `break` or `return`, you can't give a value to `continue`",
        )
    }

    fn pub_not_top_level(s: Span, d: Diagnostic) -> Diagnostic {
        d.highlight(
            s,
            "this binding is not at the top level scope, and is marked `pub`",
        )
        .info("only top-level bindings can be exported with `pub`")
    }

    fn import_not_top_level(s: Span, d: Diagnostic) -> Diagnostic {
        d.highlight(s, "this import is not at the top level scope")
            .info("import statements must be on the top-level")
    }

    fn shadow_export(s: Span, p: Span, d: Diagnostic) -> Diagnostic {
        d.highlight(
            p,
            "this is top-level binding is exported, and cannot be shadowed",
        )
        .highlight(s, "this binding is shadowing the previous one")
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

    fn too_many_functions(s: Span, d: Diagnostic) -> Diagnostic {
        d.highlight(s, "this function is the culprit").info(format!(
            "each module can only have {} function definitions",
            Module::MAX_FUNCTIONS - 1,
        ))
    }

    fn too_many_locals(s: Span, d: Diagnostic) -> Diagnostic {
        d.highlight(s, "this binding is the culprit")
            .info(format!(
                "each module can only have {} local bindings",
                Function::MAX_BINDINGS - 1,
            ))
            .info("both function parameters and `let` bindings count")
    }

    fn too_many_exports(s: Span, d: Diagnostic) -> Diagnostic {
        d.highlight(s, "this binding is the culprit").info(format!(
            "each module can only have {} top-level exported bindings",
            Function::MAX_BINDINGS - 1,
        ))
    }

    fn too_many_imports(s: Span, d: Diagnostic) -> Diagnostic {
        d.highlight(s, "this import is the culprit").info(format!(
            "each module can only have {} imports.",
            Function::MAX_BINDINGS - 1,
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
