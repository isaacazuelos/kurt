//! Value type "classes".
//!
//! Our allocating runtime values are implemented in a pretty unsafe way, to get
//! the memory layout we want.
//!
//! To try and keep some things clearer, object-oriented terms are used, even
//! though we don't have a real object system.

// IDEA: Write a macro that takes a struct definition and makes it a class by
//       adding the #[repr], `base: Object` field, and tests for object
//       assumptions?

use std::{any::TypeId, fmt::Debug};

use crate::memory::{trace::Trace, Object};

/// Each of our runtime types must implement this trait to allow for proper
/// resource management by the runtime.
///
/// # Note
///
/// TODO: Why do we need the 'static here?
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
pub(crate) trait Class: 'static + Debug + Sized + Trace {
    /// View our value as an [`Object`].
    fn upcast(&self) -> &Object {
        unsafe { std::mem::transmute(self) }
    }

    /// Get the [`TypeId`] of the value.
    ///
    /// # Note
    ///
    /// Rather than computing this with the usual Rust [`std::any`] mechanisms,
    /// it retrieves the value it saved when creating the value.
    fn concrete_type_id(&self) -> TypeId {
        self.upcast().concrete_type_id()
    }
}
