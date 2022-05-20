use crate::{index::Index, internal::local::Local};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Capture {
    index: Index<Local>,
    is_local: bool,
}

impl Capture {
    pub fn new(index: Index<Local>, is_local: bool) -> Capture {
        Capture { index, is_local }
    }

    /// Get the capture's index.
    pub fn index(&self) -> Index<Local> {
        self.index
    }

    /// Get the capture's is local.
    pub fn is_local(&self) -> bool {
        self.is_local
    }
}
