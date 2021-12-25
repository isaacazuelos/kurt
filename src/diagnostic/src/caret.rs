//! Caret
//!
//! A [`Caret`] is a line and column number in plain text, i.e. where a caret is
//! in the source text.

use std::fmt;

/// A location in some input stream or document.
///
/// Carets are zero-indexed, and with the cursor before the first character. So
/// `Caret::new(0, 0)` is with the caret at the beginning of the document,
/// typically 0 bytes into some input.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Caret {
    line: u32,
    column: u32,
}

impl Caret {
    /// Create a new [`Caret`], from a line and column number. These are
    /// 0-indexed.
    pub fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }

    /// The line the caret in on.
    pub fn line(self) -> u32 {
        self.line
    }

    /// The column of the caret.
    pub fn column(self) -> u32 {
        self.column
    }

    /// Increment a caret by a character. The only character that increments the
    /// line count is `\n`, which should work on Windows as the `\r\n` return
    /// sequence ends with the `\n` byte.
    ///
    /// This counts [`char`]s i.e. unicode code points, which means some things
    /// which span multiple code points (such as some emoji) might increment the
    /// column by more than one. This is sometimes intuitively 'wrong' but
    /// matches the behaviour of Emacs, Sublime, Vim (sort of), and VS Code.
    /// Given that these are to help you navigate in an editor, I think that's
    /// probably the right call.
    pub fn increment(&mut self, c: char) {
        match c {
            '\n' => self.line += 1,
            _ => self.column += 1,
        }
    }
}

impl fmt::Display for Caret {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.column)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn caret_order() {
        let l = Caret::new(2, 200);
        let r = Caret::new(10, 100);
        assert!(l < r);
    }

    #[test]
    fn caret_unicode() {
        let mut caret = Caret::new(0, 0);

        for c in "🤦🏼‍♀️".chars() {
            caret.increment(c);
        }

        assert_eq!(caret.column(), 5);
    }
}
