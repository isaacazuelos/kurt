//! Our garbage collected pointer type.
//!
//! These are cheap to copy around, and let our runtime values form reference
//! cycles in arbitrary and dynamic ways.

use std::{
    any::TypeId,
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    ptr::{addr_of_mut, NonNull},
};

use crate::memory::Managed;

use super::{allocator::InitWith, AnyManaged, Header, Tricolour};

#[repr(C)] // So we know where the header will be in the struct.
pub struct GcCell<T: Managed + ?Sized> {
    header: UnsafeCell<Header>,
    value: T,
}

impl<T: Managed + Default> GcCell<T> {
    /// Create a new GcCell. This isn't really safe to use as such anywhere
    /// other than in making a [`Gc`] pointer, which is why this isn't `pub` in
    /// any way.
    fn new() -> GcCell<T> {
        GcCell {
            header: UnsafeCell::new(Header::new::<T>()),
            value: T::default(),
        }
    }
}

/// A garbage collected pointer.
pub struct Gc<T: Managed> {
    ptr: NonNull<GcCell<T>>,
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
        unsafe { &self.ptr.as_ref().value }
    }
}

impl<T: Managed> DerefMut for Gc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut self.ptr.as_mut().value }
    }
}

impl<T: Managed> Gc<T> {
    /// Create a new Gc pointer.
    ///
    /// # Safety
    ///
    /// This is generally only safe to do:
    ///
    /// - If the pointer came from calling [`Gc::as_cell_ptr`].
    /// - If you're a garbage collected heap and creating a new [`Gc`] pointer
    ///   which you're taking responsibility for tracking.
    pub(crate) unsafe fn from_cell_ptr(ptr: NonNull<GcCell<T>>) -> Gc<T> {
        Gc { ptr }
    }

    /// View this as a pointer to the GC cell.
    pub(crate) fn as_cell_ptr(ptr: Gc<T>) -> NonNull<GcCell<T>> {
        ptr.ptr
    }

    /// Attempt to downcast an [`AnyManaged`] gc pointer to a specific
    /// [`Managed`] type.
    ///
    /// This uses the [`Header`] type tag to verify the cast is okay, and is
    /// checking for equality. That means we don't have sub-typing beyond the
    /// one top type, this isn't really OO-like inheritance.
    pub fn downcast(ptr: Gc<AnyManaged>) -> Option<Gc<T>> {
        if TypeId::of::<T>() == Gc::header(ptr).tag() {
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

impl<T: Managed + Default> Gc<T> {
    pub(crate) unsafe fn init(ptr: NonNull<GcCell<T>>) -> Gc<T> {
        ptr.as_ptr().write(GcCell::new());

        Gc { ptr }
    }
}

impl<T> Gc<T>
where
    T: Managed,
{
    pub(crate) unsafe fn init_with<A>(ptr: NonNull<GcCell<T>>, args: A) -> Gc<T>
    where
        T: InitWith<A>,
    {
        // Write the header.
        (*ptr.as_ptr()).header.get().write(Header::new::<T>());

        // Create a NonNull to the value only.
        let value_ptr =
            NonNull::new_unchecked(addr_of_mut!((*ptr.as_ptr()).value));

        // Initialize the value.
        T::init_with(value_ptr, args);

        Gc::from_cell_ptr(ptr)
    }
}

impl Gc<AnyManaged> {
    /// Get a copy of the allocated object's [`Header`]. Note this isn't
    /// mutable. Mutating a header is a _really_ bad idea.
    fn header(object: Gc<AnyManaged>) -> Header {
        unsafe { *object.ptr.as_ref().header.get() }
    }

    /// Get a mutable reference to the managed value's [`Header`].
    ///
    /// # Safety
    ///
    /// The garbage collector stores it's metadata for a value in this header,
    /// so modifying it incorrectly could either create a leak, or cause a live
    /// value to be deallocated.
    unsafe fn header_mut(object: &mut Gc<AnyManaged>) -> &mut Header {
        object.ptr.as_ref().header.get().as_mut().unwrap()
    }

    /// The current GC [`TriColour`] of this value.
    ///
    /// This is safe since it's not mutable, this just lets you know what it is
    /// at this moment.
    pub(crate) fn colour(object: Gc<AnyManaged>) -> Tricolour {
        Gc::header(object).colour
    }

    /// The current GC [`TriColour`] of this value.
    ///
    /// # Safety
    ///
    /// Setting a object to another colour could create either a leak or a
    /// use-after-free by making the collector do the wrong thing.
    pub(crate) unsafe fn set_colour(
        mut object: Gc<AnyManaged>,
        colour: Tricolour,
    ) {
        Gc::header_mut(&mut object).colour = colour;
    }
}
