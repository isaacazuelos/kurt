//! A highlight is a reference to a span in the source code with some note about
//! that span.
//!
//! Exactly how this is presented to the user depends on the context.

use crate::Span;

/// A section of the code in the window which will be for-certain be presented,
/// and a note to present alongside the highlighted region.
#[derive(Debug)]
pub(crate) struct Highlight {
    span: Span,
    note: Option<String>,
}

impl Highlight {
    /// Create a new highlighted span of source code.
    pub(crate) fn new(span: Span, note: String) -> Highlight {
        Highlight {
            span,
            note: Some(note),
        }
    }

    /// Get a the highlight's span.
    pub(crate) fn get_span(&self) -> Span {
        self.span
    }

    /// Get a the highlight's note.
    pub(crate) fn get_note(&self) -> Option<&str> {
        self.note.as_deref()
    }
}
