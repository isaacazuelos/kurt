//! Identifiers, things which are used as names at binding sites.

use std::fmt;
use syntax::Identifier;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug)]
pub struct Id(String);

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> From<&Identifier<'a>> for Id {
    fn from(identifier: &Identifier<'a>) -> Id {
        Id(identifier.as_str().nfc().collect())
    }
}
