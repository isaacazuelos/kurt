//! The severity of of some diagnostic.

use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Level {
    Error,
}

impl Level {
    /// The name of the level ready to be shown to users.
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Level::Error => "error",
        }
    }
}

impl Default for Level {
    fn default() -> Self {
        Level::Error
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
