//! Constant values which appear in the source code. Things like strings,
//! numbers, keywords, etc. but not collections.

use std::fmt;

use syntax::Literal;

// very incomplete
#[derive(Debug)]
pub enum Constant {
    Unit,
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Constant::Unit => write!(f, "()"),
        }
    }
}

impl<'a> From<&Literal<'a>> for Constant {
    fn from(_literal: &Literal<'a>) -> Self {
        todo!()
    }
}
