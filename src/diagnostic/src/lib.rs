//! Diagnostics - user-readable messages

pub mod caret;
pub mod span;

pub use self::{span::Span, caret::Caret};