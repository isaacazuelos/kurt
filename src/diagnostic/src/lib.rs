//! Diagnostics - user-readable messages

mod caret;
mod diagnostic;
mod diagnostic_coordinator;
mod emitter;
mod highlight;
mod input_coordinator;
mod level;
mod message;
mod span;

pub use self::{
    caret::Caret,
    diagnostic::Diagnostic,
    diagnostic_coordinator::DiagnosticCoordinator,
    input_coordinator::{InputCoordinator, InputId},
    span::Span,
};
