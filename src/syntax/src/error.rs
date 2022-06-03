//! Syntax Errors
//!
//! This differs from [`parser::Error`] because these are specific to the syntax
//! we're parsing.

use diagnostic::{Caret, Diagnostic, Span};

#[derive(Debug, Clone, Copy)]
pub enum Error {
    BindingNoReserved(Span),
    BindingNoEquals(Span, bool, Span),

    BlockNoOpen(Span),
    BlockNoClose(Span, Span),

    CallNoOpen(Span, Span),
    CallNoClose(Span, Span),

    EarlyExitNoReservedWord(Span),

    ExpressionInvalidStart(Span),

    IfNoReserved(Span),
    IfNoElse(Span, Span),

    FunctionNoOpen(Span),
    FunctionNoClose(Span, Span),
    FunctionNoArrow(Span, Span),
    FunctionNoBody(Span, Span),

    GroupingNoOpen(Span),
    GroupingNoClose(Span, Span),

    IdentifierMissing(Span),

    ListNoOpen(Span),
    ListNoClose(Span, Span),

    UnitNoOpen(Span),
    UnitNoClose(Span, Span),

    KeywordNoSpace(Span, Span),
    KeywordNoColon(Span),
    KeywordNoName(Span, Span),

    SubscriptNoOpen(Span),
    SubscriptNoClose(Span, Span),

    TopLevelUnusedInput(Span, Span),
}

impl From<Error> for parser::Error<Error> {
    fn from(val: Error) -> Self {
        parser::Error::Syntax(val)
    }
}

impl From<Error> for Diagnostic {
    fn from(e: Error) -> Self {
        match e {
            Error::BindingNoReserved(span) => Error::binding_no_reserved(span),

            Error::BindingNoEquals(r, l, f) => {
                Error::binding_no_equals(r, l, f)
            }

            Error::BlockNoOpen(span) => Error::block_no_open(span),

            Error::BlockNoClose(open, found) => {
                Error::block_no_close(open, found)
            }

            Error::CallNoOpen(target, span) => {
                Error::call_no_open(target, span)
            }

            Error::CallNoClose(open, found) => {
                Error::call_no_close(open, found)
            }

            Error::EarlyExitNoReservedWord(span) => {
                Error::early_exit_no_reserved(span)
            }

            Error::ExpressionInvalidStart(span) => {
                Error::expression_invalid_start(span)
            }

            Error::IfNoReserved(span) => Error::if_no_reserved(span),

            Error::IfNoElse(reserved, span) => {
                Error::if_no_else(reserved, span)
            }

            Error::FunctionNoOpen(span) => Error::fn_no_open(span),

            Error::FunctionNoArrow(params, found) => {
                Error::fn_no_arrow(params, found)
            }

            Error::FunctionNoClose(open, found) => {
                Error::fn_no_close(open, found)
            }

            Error::FunctionNoBody(arrow, found) => {
                Error::fn_no_body(arrow, found)
            }

            Error::GroupingNoOpen(span) => Error::group_no_open(span),

            Error::GroupingNoClose(open, found) => {
                Error::group_no_close(open, found)
            }

            Error::IdentifierMissing(span) => Error::identifier_missing(span),

            Error::ListNoOpen(span) => Error::list_no_open(span),

            Error::ListNoClose(open, found) => {
                Error::list_no_close(open, found)
            }

            Error::UnitNoOpen(span) => Error::unit_no_open(span),

            Error::UnitNoClose(open, found) => {
                Error::unit_no_close(open, found)
            }

            Error::KeywordNoSpace(colon, name) => {
                Error::keyword_no_space(colon, name)
            }

            Error::KeywordNoColon(span) => Error::keyword_no_colon(span),

            Error::KeywordNoName(colon, found) => {
                Error::keyword_no_name(colon, found)
            }

            Error::SubscriptNoOpen(span) => Error::subscript_no_open(span),

            Error::SubscriptNoClose(open, found) => {
                Error::subscript_no_close(open, found)
            }

            Error::TopLevelUnusedInput(prev, found) => {
                Error::top_level_unused_input(prev, found)
            }
        }
    }
}

impl Error {
    pub fn span(&self) -> Span {
        *match self {
            Error::BindingNoReserved(s) => s,
            Error::BindingNoEquals(_, _, s) => s,
            Error::BlockNoOpen(s) => s,
            Error::BlockNoClose(_, s) => s,
            Error::CallNoOpen(_, s) => s,
            Error::CallNoClose(_, s) => s,
            Error::EarlyExitNoReservedWord(s) => s,
            Error::ExpressionInvalidStart(s) => s,
            Error::IfNoReserved(s) => s,
            Error::IfNoElse(_, s) => s,
            Error::FunctionNoOpen(s) => s,
            Error::FunctionNoClose(_, s) => s,
            Error::FunctionNoArrow(_, s) => s,
            Error::FunctionNoBody(_, s) => s,
            Error::GroupingNoOpen(s) => s,
            Error::GroupingNoClose(_, s) => s,
            Error::IdentifierMissing(s) => s,
            Error::ListNoOpen(s) => s,
            Error::ListNoClose(_, s) => s,
            Error::UnitNoOpen(s) => s,
            Error::UnitNoClose(_, s) => s,
            Error::KeywordNoSpace(_, s) => s,
            Error::KeywordNoColon(s) => s,
            Error::KeywordNoName(_, s) => s,
            Error::SubscriptNoOpen(s) => s,
            Error::SubscriptNoClose(_, s) => s,
            Error::TopLevelUnusedInput(s, _) => s,
        }
    }

    pub fn start(&self) -> Caret {
        self.span().start()
    }

    pub fn end(&self) -> Caret {
        self.span().end()
    }

    fn binding_no_reserved(span: Span) -> Diagnostic {
        Diagnostic::new("a binding starts with `let` or `var`")
            .location(span.start())
            .highlight(span, "this isn't `let` or `var`")
    }

    fn binding_no_equals(
        reserved: Span,
        is_let: bool,
        found: Span,
    ) -> Diagnostic {
        let kind = if is_let { "let" } else { "var" };
        Diagnostic::new(format!("a `{}` is missing it's `=`", kind))
            .location(found.start())
            .highlight(reserved, "we started declaring a new variable here")
            .highlight(found, "but there's no `=` here to assign it a value")
            .info("all new variables need to be given a value")
    }

    fn block_no_open(span: Span) -> Diagnostic {
        Diagnostic::new("expected a `{` to start a block")
            .location(span.start())
            .highlight(span, "this isn't a `{`")
    }

    fn block_no_close(open: Span, found: Span) -> Diagnostic {
        Diagnostic::new("expected a `}` to close a block")
            .location(found.start())
            .highlight(open, "because of this `{` we know we're in a block")
            .highlight(found, "but this isn't a `}` to end it")
    }

    fn call_no_open(target: Span, span: Span) -> Diagnostic {
        Diagnostic::new("expected a `(` to start a function call")
            .location(span.start())
            .highlight(target, "when calling this expression")
            .highlight(
                span,
                "this would need to be a `(` to start the argument list",
            )
    }

    fn call_no_close(open: Span, span: Span) -> Diagnostic {
        Diagnostic::new("expected a `)` to finish a function call")
            .location(span.start())
            .highlight(open, "a function call's argument list started here")
            .highlight(span, "but this isn't a `)` to end it")
    }

    fn early_exit_no_reserved(span: Span) -> Diagnostic {
        Diagnostic::new(
            "expected one of `return`, `yield`, `break`, or `continue`.",
        )
        .location(span.start())
        .highlight(span, "expected here")
    }

    fn expression_invalid_start(span: Span) -> Diagnostic {
        Diagnostic::new("expected an expression here")
            .location(span.start())
            .highlight(span, "this is not the start of an expression")
    }

    fn if_no_reserved(span: Span) -> Diagnostic {
        Diagnostic::new("expected an `if` here")
            .location(span.start())
            .highlight(span, "this isn't `if`")
    }

    fn if_no_else(reserved: Span, span: Span) -> Diagnostic {
        Diagnostic::new("an `if` expected an `else`")
            .location(span.start())
            .highlight(reserved, "this `if` needs an `else`")
            .highlight(span, "the `else` should be here")
            .info(
                "when an `if` is used as an expression, \
                it needs to have an `else` part",
            )
    }

    fn fn_no_open(span: Span) -> Diagnostic {
        Diagnostic::new("a `(` was expected when parsing a function")
            .location(span.start())
            .highlight(
                span,
                "this isn't the `(` that starts a the list of parameters",
            )
    }

    fn fn_no_arrow(params: Span, found: Span) -> Diagnostic {
        Diagnostic::new("a function is missing it's `=>`")
            .location(found.start())
            .highlight(params, "the function started here expected a `=>`")
            .highlight(found, "the `=>` should be here")
    }

    fn fn_no_close(open: Span, found: Span) -> Diagnostic {
        Diagnostic::new("a function's parameter list was never closed")
            .location(found.start())
            .highlight(open, "the parameter list started here expected a `)`")
            .highlight(found, "the `)` should be here")
    }

    fn fn_no_body(start: Span, found: Span) -> Diagnostic {
        Diagnostic::new("a function is missing its body expression")
            .location(found.start())
            .highlight(start, "the function started here")
            .highlight(found, "expected an expression here for its body")
    }

    fn group_no_open(span: Span) -> Diagnostic {
        Diagnostic::new("expected to see a `(` used for grouping")
            .location(span.start())
            .highlight(span, "expected a `(` to be used for grouping")
    }

    fn group_no_close(open: Span, found: Span) -> Diagnostic {
        Diagnostic::new("expected to see a `)`")
            .location(found.start())
            .highlight(open, "to match this opening one")
            .highlight(found, "but instead we have this")
    }

    fn identifier_missing(span: Span) -> Diagnostic {
        Diagnostic::new("an identifier was expected")
            .location(span.start())
            .highlight(span, "an identifier was expected here")
            .help(
                "an identifier is a name for a value, \
                such as the `x` in `let x = 1;`",
            )
    }

    fn list_no_open(span: Span) -> Diagnostic {
        Diagnostic::new("expected to see a `[` to start a list")
            .location(span.start())
            .highlight(span, "expected a `[` here")
    }

    fn list_no_close(open: Span, found: Span) -> Diagnostic {
        Diagnostic::new("a list is missing it's `]`")
            .location(found.start())
            .highlight(open, "the list started here")
            .highlight(found, "expected a `]` here")
    }

    fn unit_no_open(span: Span) -> Diagnostic {
        Diagnostic::new("expected the `(` for a `()` value")
            .location(span.start())
            .highlight(span, "a `(` was expected here")
    }

    fn unit_no_close(open: Span, found: Span) -> Diagnostic {
        Diagnostic::new("a `()` is missing it's `)`")
            .location(found.start())
            .highlight(open, "it started here")
            .highlight(found, "and expected a `)` here")
    }

    fn keyword_no_space(colon: Span, name: Span) -> Diagnostic {
        let whitespace = Span::new(colon.end(), name.start());
        Diagnostic::new("a keyword can't have whitespace in it")
            .location(colon.end())
            .highlight(whitespace, "remove this whitespace")
    }

    fn keyword_no_colon(span: Span) -> Diagnostic {
        Diagnostic::new("a keyword has to start with a colon")
            .location(span.start())
            .highlight(span, "this isn't a keyword")
    }

    fn keyword_no_name(colon: Span, found: Span) -> Diagnostic {
        Diagnostic::new("a keyword is missing it's name")
            .location(colon.end())
            .highlight(
                colon,
                "a name for the keyword should begin after the `:`",
            )
            .highlight(found, "instead we found this")
            .info("there's no way to make an 'empty' keyword")
    }

    fn subscript_no_open(span: Span) -> Diagnostic {
        Diagnostic::new("expected a `[` to start indexing a value")
            .location(span.start())
            .highlight(span, "a `[` was expected here")
    }

    fn subscript_no_close(open: Span, found: Span) -> Diagnostic {
        Diagnostic::new("a subscript is missing it's `]`")
            .location(found.start())
            .highlight(open, "the subscript started here")
            .highlight(found, "expected a `]` here")
    }

    fn top_level_unused_input(prev: Span, issue: Span) -> Diagnostic {
        Diagnostic::new("a statement didn't end where expected")
            .location(prev.end())
            .highlight(prev, "should there be a semicolon after this?")
            .highlight(
                issue,
                "this doesn't look like part of the previous statement",
            )
    }
}
