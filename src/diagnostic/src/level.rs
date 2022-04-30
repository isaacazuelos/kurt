//! The severity or significance of some diagnostic information.

use std::fmt;

/// A level indicates the severity or relevance of some piece of information.
#[derive(Debug, Clone, Copy)]
pub enum Level {
    Error,
    Info,
    Help,
    // If adding levels with non-ascii `name`s, you'll need to update how
    // FancyEmitter prints them.
}

impl Level {
    /// The name of the level ready to be shown to users.
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Level::Error => "error",
            Level::Info => "info",
            Level::Help => "help",
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
