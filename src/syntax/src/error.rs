//! Syntax Errors
//!
//! This differs from [`parser::Error`] because these are specific to the syntax
//! we're parsing.

use diagnostic::Diagnostic;

#[derive(Debug)]
pub enum Error {
    BindingNoReserved,
    BindingNoEquals,

    BlockNoOpen,
    BlockNoClose,

    CallNoOpen,
    CallNoClose,

    ExpressionInvalidStart,

    IfNoReserved,
    IfNoElse,

    FunctionNoOpen,
    FunctionNoClose,
    FunctionNoArrow,

    GroupingNoOpen,
    GroupingNoClose,

    IdentifierMissing,

    ListNoOpen,
    ListNoClose,

    UnitNoOpen,
    UnitNoClose,

    KeywordNoSpace,
    KeywordNoColon,
    KeywordNoName,

    SubscriptNoOpen,
    SubscriptNoClose,
}

impl From<Error> for parser::Error<Error> {
    fn from(val: Error) -> Self {
        parser::Error::Syntax(val)
    }
}

impl From<Error> for Diagnostic {
    fn from(e: Error) -> Self {
        Diagnostic::new("unfinished syntax error")
            .info(format!("the raw error is {:?}", e))
    }
}
