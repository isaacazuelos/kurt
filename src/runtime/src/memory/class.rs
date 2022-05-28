//! Value type "classes".
//!
//! Our allocating runtime values are implemented in a pretty unsafe way, to get
//! the memory layout we want.
//!
//! To try and keep some things clearer, object-oriented terms are used, even
//! though we don't have a real object system.

use std::fmt::Debug;

use crate::memory::trace::Trace;

/// Class IDs are used as type tags.
///
/// In the past Rust's [`Any`] type id was used, but I kept missing places where
/// I was matching on it. Instead, now we can use these (and [`dispatch!`][1])
/// to keep things exhaustive and safe.
///
/// [1]: crate::memory::object::dispatch!
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ClassId {
    CaptureCell,
    Closure,
    Keyword,
    List,
    Module,
    Prototype,
    String,
}

impl ClassId {
    pub fn name(&self) -> &'static str {
        match self {
            ClassId::CaptureCell => "CaptureCell",
            ClassId::Closure => "Closure",
            ClassId::Keyword => "Keyword",
            ClassId::List => "List",
            ClassId::Module => "Module",
            ClassId::Prototype => "Prototype",
            ClassId::String => "String",
        }
    }
}

/// Each of our runtime types must implement this trait to allow for proper
/// resource management by the runtime.
///
/// # Safety
///
/// Each type which implements [`Class`] must do the following:
///
/// 1. It must be a struct which is [`#[repr(C, align(8))]`][repr].
///
/// 2. It must start with a felid of type [`Object`].
///
/// This makes Rust promise that the object is laid out in memory consistently
/// and with the [`Object`] first, so we can downcast and have all the object
/// metadata in the right place.
///
/// [repr]: https://doc.rust-lang.org/nomicon/other-reprs.html#reprc
pub trait Class: Debug + Sized + Trace {
    /// The [`ClassId`] that's unique to objects of this class.
    const ID: ClassId;

    /// The address of this object as usize, like [python's `id`][id]. The same
    /// caveat applies about object lifetimes.
    ///
    /// [id]: https://docs.python.org/2/library/functions.html#id
    fn identity(&self) -> usize {
        self as *const Self as usize
    }
}
