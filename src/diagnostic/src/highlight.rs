//! A highlight is a reference to a span in the source code with some note about
//! that span.
//!
//! Exactly how this is presented to the user depends on the context.

use crate::Span;

/// A section of the code in the window which will be for-certain be presented,
/// and with some indication of it's importance, and a note to present alongside
/// the highlighted region.
#[derive(Debug)]
pub struct Highlight {
    span: Span,
    note: Option<String>,
}

impl Highlight {
    /// Create a new highlighted span of source code.
    pub fn new(span: Span, note: impl Into<String>) -> Highlight {
        Highlight {
            span,
            note: Some(note.into()),
        }
    }

    /// Get a the highlight's span.
    pub fn span(&self) -> Span {
        self.span
    }

    /// Get a the highlight's note.
    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }
}
