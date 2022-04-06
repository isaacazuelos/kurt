//! Lisp-style `:foo` keywords

use std::fmt::{self, Debug};

use crate::memory::{
    class::Class,
    string::String,
    trace::{Trace, WorkList},
    InitFrom,
};

/// In many dynamic languages, strings are used both to represent text and also
/// to serve as token values for things like enumerations and dictionaries.
///
/// We have strings, but use [`Keyword`]s for these second cases.
#[repr(transparent)]
pub struct Keyword {
    string: String,
}

impl Class for Keyword {}

impl Trace for Keyword {
    fn enqueue_gc_references(&self, _: &mut WorkList) {}
}

impl InitFrom<&str> for Keyword {
    fn size(arg: &&str) -> usize {
        String::size(arg)
    }

    /// # Safety
    ///
    /// Perhaps surprisingly, this is safe according to the contract for
    /// [`Class`] and [`Object`][obj]. Since the [`Object`][obj] at the start of
    /// any [`Class`] is initialized before any class-specific initialization in
    /// [`init`][ci], and that initialization can't change the base object, we
    /// know [`String::init`] can't mess it up.
    ///
    /// [obj]: super::Object
    unsafe fn init(ptr: *mut Self, args: &str) {
        String::init(ptr as *mut String, args);
    }
}

impl Keyword {
    /// View the Keyword as a Rust &str.
    pub fn as_str(&self) -> &str {
        self.string.as_str()
    }
}

impl Debug for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, ":{}", self.string.as_str())
    }
}
