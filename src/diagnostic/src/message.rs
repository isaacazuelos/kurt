//! A generic message

use std::fmt;

use crate::level::Level;

/// A message is some information about an issue.
#[derive(Debug)]
pub(crate) struct Message {
    level: Level,
    text: String,
}

impl Message {
    /// Create a new message.
    pub(crate) fn new(level: Level, text: String) -> Message {
        let text = text;
        Message { level, text }
    }

    /// The message's [`Level`] tells us what kind of information this is.
    ///
    /// This is often relevant when deciding how the message should be shown.
    pub(crate) fn get_level(&self) -> Level {
        self.level
    }

    /// Set the level of the message to something else.
    pub(crate) fn set_level(&mut self, level: Level) {
        self.level = level;
    }

    /// The text of the message.
    pub(crate) fn get_text(&self) -> &str {
        &self.text
    }

    /// Set the text of the message to something else.
    pub(crate) fn set_text(&mut self, text: String) {
        self.text = text;
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.level, self.text)
    }
}
