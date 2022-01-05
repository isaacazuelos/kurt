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
    pub unsafe fn from_ptr(ptr: NonNull<T>) -> Gc<T> {
        Gc { ptr }
    }

    pub fn as_ptr(self) -> *mut T {
        self.ptr.as_ptr()
    }

    pub fn downcast<S: Managed>(ptr: Gc<T>) -> Option<Gc<S>> {
        if TypeId::of::<S>() == ptr.header().tag() {
            Some(unsafe { std::mem::transmute(ptr) })
        } else {
            None
        }
    }

    pub fn as_any(self) -> Gc<AnyManaged> {
        Gc {
            ptr: unsafe { std::mem::transmute(self.ptr) },
        }
    }
}

impl<T: Managed> Gc<T> {
    fn header(self) -> Header {
        let any = Gc::as_any(self);
        any.deref().header()
    }
}
