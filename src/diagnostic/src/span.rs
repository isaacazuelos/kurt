//! Spans - selections in source code
//!
//! Each piece of input takes up some space, it's not just a point like a
//! [`Caret`], but a selection with a beginning and end which might span across
//! lines or even be empty.

use std::cmp::{max, min};
use std::fmt;

use crate::caret::Caret;

/// A contiguous span between two carets in a source document. The span is
/// _inclusive_, in that the span of "the" is between the `|`s in "|the|", i.e.
/// it's between 0 and _3_, even though `e` is character _2_.
#[derive(Clone, Debug, Default, Copy, Eq, Hash, PartialEq)]
pub struct Span {
    start: Caret,
    end: Caret,
}

impl Span {
    /// Return a new span over the two carets.
    ///
    /// The carets do not need to be sorted.
    pub fn new(l1: Caret, l2: Caret) -> Self {
        let start = min(l1, l2);
        let end = max(l1, l2);
        Self { start, end }
    }

    /// Where the span starts.
    pub fn start(&self) -> Caret {
        self.start
    }

    /// Where the span ends.
    pub fn end(&self) -> Caret {
        self.end
    }

    /// The intersection of two ranges, if they overlap, and [`None`] if they do
    /// not overlap.
    pub fn intersection(&self, other: Span) -> Option<Span> {
        let lower_end = min(self.end(), other.end());
        let higher_start = max(self.start(), other.start());

        if lower_end > higher_start {
            Some(Span::new(higher_start, lower_end))
        } else {
            None
        }
    }
}

impl ::std::ops::Add for Span {
    type Output = Self;

    /// Adding spans returns a new span which covers all of each of the spans
    /// given (and any characters in between.)
    ///
    /// This operation commutes, but has no identity.
    fn add(self, other: Self) -> Self {
        let start = min(self.start, other.start);
        let end = max(self.end, other.end);
        Self::new(start, end)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // I'm using an en-dash here, as that's technically correct. But it
        // may be unwise to hard-code in non-ascii. We'll see.
        write!(f, "{}–{}", self.start, self.end)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn span_caret_order() {
        let l = Caret::new(2, 200);
        let r = Caret::new(10, 100);
        assert_eq!(Span::new(l, r), Span::new(r, l));
    }
    #[test]
    fn span_over() {
        let l = Span::new(Caret::new(2, 200), Caret::new(10, 100));
        let r = Span::new(Caret::new(0, 0), Caret::new(0, 100));
        assert_eq!(l + r, Span::new(Caret::new(0, 0), Caret::new(10, 100)))
    }
    #[test]
    fn inner_span() {
        let l = Span::new(Caret::new(0, 0), Caret::new(100, 100));
        let r = Span::new(Caret::new(0, 0), Caret::new(10, 10));
        assert_eq!(l + r, l);
    }
}
