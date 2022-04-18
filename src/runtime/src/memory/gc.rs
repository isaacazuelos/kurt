//! Pointers to values managed by the garbage collector.

use std::ptr::NonNull;

use crate::memory::Object;

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
