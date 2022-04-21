//! Lisp-style `:foo` keywords

use std::fmt::{self, Debug};

use crate::{
    memory::{
        class::{Class, ClassId},
        string::String,
        trace::{Trace, WorkList},
        InitFrom,
    },
    primitives::PrimitiveOperations,
};

/// In many dynamic languages, strings are used both to represent text and also
/// to serve as token values for things like enumerations and dictionaries.
///
/// We have strings, but use [`Keyword`]s for these second cases.
#[repr(transparent)]
pub struct Keyword {
    string: String,
}

impl Class for Keyword {
    const ID: ClassId = ClassId::Keyword;
}

impl PartialEq for Keyword {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialOrd for Keyword {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.string.partial_cmp(&other.string)
    }
}

impl Trace for Keyword {
    fn enqueue_gc_references(&self, _: &mut WorkList) {}
}

impl InitFrom<&str> for Keyword {
    fn extra_size(arg: &&str) -> usize {
        String::extra_size(arg)
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

impl PrimitiveOperations for Keyword {
    fn type_name(&self) -> &'static str {
        "Keyword"
    }

    fn cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(self, other)
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
