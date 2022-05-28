//! Pointers to values managed by the garbage collector.

use std::fmt::{self, Debug, Formatter};
use std::ops::Deref;
use std::{marker::PhantomData, ptr::NonNull};

use crate::memory::{Class, Object};

#[repr(transparent)]
pub struct Gc<T: Class> {
    any: GcAny,
    class: PhantomData<T>,
}

impl<T: Class> Gc<T> {
    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn deref_mut(&self) -> &mut T {
        // yikes
        self.any.ptr.cast().as_mut()
    }
}

impl<T: Class> Clone for Gc<T> {
    fn clone(&self) -> Self {
        Gc {
            any: self.any,
            class: self.class,
        }
    }
}

impl<T: Class> Debug for Gc<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self.any)
    }
}

impl<T: Class> Copy for Gc<T> {}

impl<T: PartialEq + Class> PartialEq for Gc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

impl<T: Class> From<Gc<T>> for GcAny {
    #[inline]
    fn from(ptr: Gc<T>) -> Self {
        unsafe { std::mem::transmute(ptr) }
    }
}

impl<T: Class> Deref for Gc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.any.ptr.cast().as_ref() }
    }
}

/// A type-erased pointer to a garbage collected value.
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

    #[inline]
    pub fn as_a<T: Class>(self) -> Option<Gc<T>> {
        if self.is_a::<T>() {
            Some(unsafe { std::mem::transmute(self) })
        } else {
            None
        }
    }

    pub(crate) unsafe fn cast_unchecked<T: Class>(self) -> Gc<T> {
        std::mem::transmute(self)
    }
}

impl Deref for GcAny {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.cast().as_ref() }
    }
}

impl Clone for GcAny {
    fn clone(&self) -> Self {
        GcAny { ptr: self.ptr }
    }
}

impl Copy for GcAny {}

impl Debug for GcAny {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self.deref())
    }
}
