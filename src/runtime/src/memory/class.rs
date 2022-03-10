//! Value type "classes".
//!
//! TODO: Explain what I mean by class, since this really aren't how OO classes
//! work.

use std::fmt::Debug;

use crate::memory::object::Tag;

use super::Object;

pub(crate) trait Class: 'static + Debug + Sized {
    /// The type tag used by this class.
    const TAG: Tag;

    fn upcast(&self) -> &Object {
        unsafe { std::mem::transmute(self) }
    }

    fn tag(&self) -> Tag {
        self.upcast().tag()
    }
}

// IDEA: Write a macro that takes a struct definition and makes it a class by
//       adding the #[repr], `base: Object` field, and tests for object
//       assumptions?
