//! Our garbage collected pointer type.
//!
//! These are cheap to copy around, and let our runtime values form reference
//! cycles in arbitrary and dynamic ways.

use std::{any::TypeId, ops::Deref, ptr::NonNull};

use crate::memory::Managed;

use super::{AnyManaged, Header};

/// A garbage collected pointer.
pub struct Gc<T: Managed> {
    ptr: NonNull<T>,
}

impl<T: Managed> Clone for Gc<T> {
    fn clone(&self) -> Self {
        Gc { ptr: self.ptr }
    }
}

impl<T: Managed> Copy for Gc<T> {}

impl<T: Managed> Deref for Gc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: Managed> Gc<T> {
    /// Create a new Gc pointer.
    ///
    /// # Safety
    ///
    /// This is generally only safe to do:
    ///
    /// - If the pointer came from calling [`Gc::as_ptr`].
    ///
    /// - If you're a garbage collected heap and creating a new [`Gc`] pointer
    ///   which you're taking responsibility for tracking.
    pub unsafe fn from_ptr(ptr: NonNull<T>) -> Gc<T> {
        Gc { ptr }
    }

    /// View this pointer as a raw mutable pointer.
    pub fn as_ptr(self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Attempt to downcast an [`AnyManaged`] gc pointer to a specific
    /// [`Managed`] type.
    ///
    /// This uses the [`Header`] type tag to verify the cast is okay, and is
    /// checking for equality. That means we don't have sub-typing beyond the
    /// one top type, this isn't really OO-like inheritance.
    pub fn downcast(ptr: Gc<AnyManaged>) -> Option<Gc<T>> {
        if TypeId::of::<T>() == ptr.header().tag() {
            Some(unsafe { std::mem::transmute(ptr) })
        } else {
            None
        }
    }

    /// Upcast this pointer into a gc pointer to [`AnyManaged`].
    ///
    /// This lets collections safely store any garbage collected value in a
    /// field, which is needed for things like tuples.
    ///
    /// # Notes
    ///
    /// See [`Gc::downcast`] for how to get back the original [`Managed`] type.
    pub fn as_any(self) -> Gc<AnyManaged> {
        Gc {
            ptr: unsafe { std::mem::transmute(self.ptr) },
        }
    }
}

impl<T: Managed> Gc<T> {
    /// Get a copy of the allocated object's [`Header`]. Note this isn't
    /// mutable. Mutating a header is a _really_ bad idea.
    fn header(self) -> Header {
        let any = Gc::as_any(self);
        any.deref().header()
    }
}
