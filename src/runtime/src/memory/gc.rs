//! Pointers to values managed by the garbage collector.

use std::{marker::PhantomData, ptr::NonNull};

use crate::memory::{Class, Object};

use super::ClassId;

#[derive(Debug)]
#[repr(transparent)]
pub struct Gc<T: Class> {
    any: GcAny,
    class: PhantomData<T>,
}

impl<T: Class> Clone for Gc<T> {
    fn clone(&self) -> Self {
        Gc {
            any: self.any,
            class: self.class,
        }
    }
}

impl<T: Class> Copy for Gc<T> {}

impl<T: Class> From<Gc<T>> for GcAny {
    #[inline]
    fn from(ptr: Gc<T>) -> Self {
        unsafe { std::mem::transmute(ptr) }
    }
}

pub struct DowncastError {
    pub from: ClassId,
    pub to: ClassId,
}

impl<T: Class> TryFrom<GcAny> for Gc<T> {
    type Error = DowncastError;

    fn try_from(any: GcAny) -> Result<Self, Self::Error> {
        if any.is_a::<T>() {
            Ok(unsafe { std::mem::transmute(any) })
        } else {
            Err(DowncastError {
                from: any.deref().class_id(),
                to: T::ID,
            })
        }
    }
}

/// A type-erased pointer to a garbage collected value.
#[derive(Debug)]
#[repr(transparent)]
pub struct GcAny {
    ptr: NonNull<Object>,
}

impl GcAny {
    /// Crate a new [`GcObj`] from a pointer. This should only be called by the
    /// heap after initializing a value.
    ///
    /// # Safety
    ///
    /// The pointed-to object must be fully initialized.
    ///
    /// This does nothing to manage the pointer, which is why it should only be
    /// called by the heap when allocating.
    #[inline]
    pub(crate) unsafe fn from_non_null(ptr: NonNull<Object>) -> GcAny {
        GcAny { ptr }
    }

    /// View the pointer as a regular Rust reference to an [`Object`].
    ///
    /// The returned reference's lifetime is inherited from `self`, so this
    /// reference will also keep the [`GcAny`] alive. Which is all to save that
    /// as long as the [`GcObj`] is kept traceable properly these references
    /// should be safe.
    #[inline]
    pub(crate) fn deref(&self) -> &Object {
        unsafe { self.ptr.as_ref() }
    }

    #[inline]
    pub fn is_a<T: Class>(self) -> bool {
        self.deref().class_id() == T::ID
    }
}

impl Clone for GcAny {
    fn clone(&self) -> Self {
        GcAny { ptr: self.ptr }
    }
}

impl Copy for GcAny {}
