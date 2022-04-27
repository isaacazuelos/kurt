//! A generic message

use std::fmt;

use crate::level::Level;

#[derive(Debug)]
pub(crate) struct Message {
    pub(crate) level: Level,
    pub(crate) text: String,
}

impl Message {
    pub(crate) fn new(level: Level, text: String) -> Message {
        let text = text.into();
        Message { level, text }
    }

    pub(crate) fn level(&self) -> Level {
        self.level
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.level, self.text)
    }
}
