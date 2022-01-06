//! Memory allocator traits.
//!
//! This are very similar to [`std::alloc::Allocator`]'s, and if that's ever
//! stable it's probably worth moving over.

use std::{alloc::Layout, ptr::NonNull};

use crate::memory::{Gc, Managed};

/// An abstraction over what we can use as an allocator for managing our memory.
///
/// The idea here is the API should be powerful enough that we can swap out
/// different allocation and collection strategies later.
///
/// # Safety
///
/// This whole endeavour is unsafe intrinsically. There's no way to use this
/// without sidestepping Rust and going into C memory error land.
///
/// Any pointer allocated must be deallocated, and any deallocated pointer must
/// not be dereferenced.
pub(crate) unsafe trait Allocator {
    /// Allocate memory as described by `layout`. Allocators _may_ allocate more
    /// memory than requested based on their supported granularity.
    ///
    /// Returns an [valid] pointer to at least as many bytes required by the
    /// `layout`, or `None` if cannot satisfy the request.
    ///
    /// [valid]: https://doc.rust-lang.org/nightly/core/ptr/index.html#safety
    ///
    /// # Safety
    ///
    /// The returned memory isn't initialized.
    fn allocate(&mut self, layout: Layout) -> Option<NonNull<u8>>;

    /// Return some memory to the system to be used again.
    ///
    /// # Safety
    ///
    /// For this to be safe the following must be true:
    ///
    /// 1. The pointer was allocated by this allocator.
    /// 2. The layout provided matches the layout of the original call to
    ///    [`allocate`][Allocator::allocate] that created the pointer.
    /// 3. There are no other live copies of the pointer.
    ///
    /// I'm be a little vague on what 'live' means above. This is on purpose --
    /// I don't know.
    unsafe fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout);

    /// Did some pointer come from this allocator?
    ///
    /// # Note
    ///
    /// Depending on how this allocator works, this might not be especially
    /// fast. This really shouldn't be relied upon whenever possible, but
    /// implementing it requires that an allocator be able to track it's
    /// allocations. It's a good sanity check, both for allocators and as
    /// something we can [`debug_assert!`].
    fn contains(&self, ptr: NonNull<u8>) -> bool;

    /// Create a new gc pointer to a `T` which was initialized with [`Default`].
    ///
    /// If the allocation fails for any reason, this will return [`None`].
    fn make<T: Managed + Default>(&mut self) -> Option<Gc<T>> {
        let layout = Layout::new::<T>();
        let ptr = self.allocate(layout)?.cast::<T>();

        // SAFETY: The requirements are met because ptr came from `allocate`,
        //         which tells us the pointer fits `T` and is valid for writes.
        unsafe { ptr.as_ptr().write(T::default()) };

        Some(unsafe { Gc::from_ptr(ptr) })
    }

    /// Create a new gc pointer to a `T` which was initialized with some
    /// argument as described by it's [`InitWith`] implementation.
    ///
    /// If the allocation fails for any reason, this will return [`None`].    
    fn make_with<T: Managed + InitWith<A>, A>(
        &mut self,
        args: A,
    ) -> Option<Gc<T>> {
        let layout = T::layout(&args)?;
        let ptr = self.allocate(layout)?.cast::<T>();

        unsafe { T::init_with(ptr, args) };

        Some(unsafe { Gc::from_ptr(ptr) })
    }
}

/// This trait provides the (very unsafe) methods we need to initialize unsized
/// managed objects.
///
/// This is basically our Managed version of [`From`], just a lot less ergonomic
/// sadly, since we _must_ to work with uninitialized memory in ugly ways here.
pub trait InitWith<Arg>: Managed {
    /// Initialize some uninitialized raw memory with some argument.
    ///
    /// # Note
    ///
    /// If you want to provide multiple values, you can make `Arg` a tuple or
    /// slice. You can also implement this with different types of `Arg` to
    /// allow creating your [`Managed`] type from different types of values.
    ///
    /// # Safety
    ///
    /// The pointer is assumed to be the the right layout, which can be computed
    /// with [`InitWith::layout`]. It's not even guaranteed to panic if this
    /// isn't met, it'll just _do bad things_.
    ///
    /// This memory is assumed to be uninitialized. Don't call this on already
    /// initialized memory or the value there will not be properly dropped.
    unsafe fn init_with(ptr: NonNull<Self>, args: Arg);

    /// The [`Layout`] needed to initialize an instance of this type with the
    /// arguments specified.
    ///
    /// This can return [`None`] if the resulting layout doesn't make sense (see
    /// [`LayoutError`][std::alloc::LayoutError] for more.)
    fn layout(args: &Arg) -> Option<Layout>;
}
