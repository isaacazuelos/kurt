//! Pointers to values managed by the garbage collector.

use std::any::TypeId;
use std::{fmt, ptr::NonNull};

use crate::memory::{string::String, Object};

/// A type-erased pointer to a garbage collected value.
#[derive(Debug)]
#[repr(transparent)]
pub struct Gc {
    ptr: NonNull<Object>,
}

impl Gc {
    /// Crate a new [`GcObj`] from a pointer. This should only be called by the
    /// heap after initializing a value.
    ///
    /// # Safety
    ///
    /// The pointed-to object must be fully initialized.
    ///
    /// This does nothing to manage the pointer, which is why it should only be
    /// called by the heap when allocating.
    pub(crate) unsafe fn from_non_null(ptr: NonNull<Object>) -> Gc {
        Gc { ptr }
    }

    /// View the pointer as a regular Rust reference to an [`Object`].
    ///
    /// The returned reference's lifetime is inherited from `self`, so this
    /// reference will also keep the [`GcObj`] alive. Which is all to save that
    /// as long as the [`GcObj`] is kept traceable properly these references
    /// should be safe.
    ///
    /// # Note
    ///
    /// This isn't [`std::ops::Deref`] because we don't want to leak [`Object`]
    /// outside the crate.
    pub(crate) fn deref(&self) -> &Object {
        unsafe { self.ptr.as_ref() }
    }
}

impl Clone for Gc {
    fn clone(&self) -> Self {
        Gc { ptr: self.ptr }
    }
}

impl Copy for Gc {}

impl PartialEq for Gc {
    fn eq(&self, other: &Self) -> bool {
        let lhs = self.deref();
        let rhs = other.deref();

        lhs == rhs
    }
}

impl fmt::Display for Gc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let obj = self.deref();

        match obj.concrete_type_id() {
            type_id if type_id == TypeId::of::<String>() => {
                let s = obj.downcast::<String>().unwrap();
                write!(f, "{}", s.as_str())
            }

            _ => {
                write!(f, "<object with unknown type_id>")
            }
        }
    }
}
