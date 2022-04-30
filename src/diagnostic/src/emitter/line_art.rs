//! Line art helpers used by different emitters.

/// Different sets of line art characters, used to draw lines and arrows for a
/// code window.
#[derive(Clone, Copy)]
pub struct LineArt {
    pub(crate) vertical: char,
    pub(crate) horizontal: char,
    pub(crate) tee: char,
    pub(crate) up: char,
    pub(crate) _right: &'static str,
    pub(crate) more: &'static str,
}

impl LineArt {
    /// Unicode line art symbols
    pub const UNICODE: LineArt = LineArt {
        vertical: '│',
        horizontal: '─',
        tee: '┬',
        up: '↑',
        _right: "→",
        more: "…",
    };

    /// ASCII line art symbols
    pub const ASCII: LineArt = LineArt {
        vertical: '|',
        horizontal: '-',
        tee: '+',
        up: '^',
        _right: "->",
        more: "...",
    };
}
