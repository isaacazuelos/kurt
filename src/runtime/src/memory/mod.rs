//! A stub for the garbage collector.
//!
//! I'm not sure what I want this to look like yet, and I've been indecisive to
//! too long. So I'm just making it a blank.
//!
//! Right now any GC value that's allocated is actually just a leaked `*mut`
//! pointer.

// TODO: We might want to make some of the [`Managed`] methods not public.
//       https://jack.wrenn.fyi/blog/private-trait-methods/

// TODO: How does allocation fail? Probably just adding a `-> Result<()>` to a
//       bunch of these. Look at Rust's [Allocator][1].
//
//       [1] https://doc.rust-lang.org/std/alloc/trait.Allocator.html

use std::{alloc::Layout, ptr::NonNull};

mod class;
mod collector;
mod gc;
mod object;

mod trace;

use crate::VirtualMachine;

pub use self::{
    class::{Class, ClassId},
    gc::{Gc, GcAny},
    object::Object,
    trace::{Trace, WorkList},
};

/// Since our [`Class`] values can be [DSTs][dst], we need a way to initialize
/// them. This trait helps us do that.
///
/// [dst]: https://doc.rust-lang.org/nomicon/exotic-sizes.html
// TODO: rename to something else because of Module::init etc,
pub(crate) trait InitFrom<A>
where
    Self: Class,
{
    /// The extra amount of space the [`Class`] needs in bytes, when initialized
    /// from the given argument. This is above whatever `size_of::<Class>` is.
    fn extra_size(arg: &A) -> usize;

    /// Initialize the pointer using the given value.
    ///
    /// # Safety
    ///
    /// When the function returns, the pointer should point to a fully
    /// initialized instance of `C`.
    ///
    /// 1. The pointer must not be null.
    ///
    /// 1. The pointer must be to uninitialized memory, other than the base
    ///    [`Object`] field, which must not be touched. All other fields should
    ///    will be overwritten without the previous value being dropped (see
    ///    [`std::ptr::write`] or [`std::ptr::copy`]) by the time we return.
    ///
    /// 1. The pointer must point to an allocation large enough, as given by
    ///    [`InitFrom::size`] for the given argument.
    unsafe fn init(ptr: *mut Self, args: A);
}

impl<T, A> InitFrom<A> for T
where
    T: From<A> + Sized + Class,
{
    fn extra_size(_arg: &A) -> usize {
        std::mem::size_of::<Self>()
    }

    unsafe fn init(ptr: *mut Self, arg: A) {
        ptr.write(Self::from(arg))
    }
}

impl VirtualMachine {
    /// Allocate a new [`Object`] and initialize it using a [`Default`]
    /// value it can be initiated from.
    #[allow(dead_code)]
    pub(crate) fn make<C, A>(&mut self) -> Gc<C>
    where
        C: Class + InitFrom<A>,
        A: Default,
    {
        self.make_from::<C, A>(A::default())
    }

    /// Allocate a new [`Object`], initializing it from the given argument.
    pub(crate) fn make_from<C, A>(&mut self, arg: A) -> Gc<C>
    where
        C: Class + InitFrom<A>,
    {
        // find the layout needed for the object.
        let extra = C::extra_size(&arg);
        let layout = VirtualMachine::object_layout_with_extra::<C>(extra);

        unsafe {
            let raw = self.allocate(layout);

            // SAFETY: For both parts of initialization, raw is uninitialized
            //         and points to something that `layout` fits in, because
            //         `raw` came from `allocate`.
            Object::init::<C>(raw, layout.size());
            C::init(raw as *mut C, arg);

            // SAFETY: We know `raw` is non-null because it came from
            //         `allocate`.
            let ptr = NonNull::new_unchecked(raw);

            // SAFETY: We know it's initialized because we just initialized it.
            let gc = GcAny::from_non_null(ptr);

            // SAFETY: We just made the GC, so we know it's not tracked.
            self.register_gc_ptr(gc);

            #[cfg(feature = "gc_trace")]
            eprintln!("initialized {:?} as {:?}", gc, gc.deref());

            gc.cast_unchecked()
        }
    }

    fn object_layout_with_extra<C: Class>(extra: usize) -> Layout {
        let align = std::mem::align_of::<C>();
        let base_size = std::mem::size_of::<C>();

        // SAFETY: Align is explicitly set for Object to 8, so it's non-zero
        //        and a power of 2. We don't check for overflows on `size`
        //        though -- should we?
        unsafe { Layout::from_size_align_unchecked(base_size + extra, align) }
    }

    /// Allocate for the given layout.
    ///
    /// # Safety
    ///
    /// The pointer returned satisfies the layout, is not null, but is not
    /// initialized either.
    ///
    /// This memory isn't tracked in any way, and it will leak.
    ///
    /// If the object is to be managed by the collector, the caller is
    /// responsible for calling [`register_gc_ptr`][crate::memory::collector]
    /// after the object is initialized into a full-fledged [`Gc`] pointer.
    ///
    /// Otherwise it's the caller's responsibility to ensure it is freed
    /// appropriately.
    unsafe fn allocate(&mut self, layout: Layout) -> *mut Object {
        self.collect_garbage();

        #[allow(clippy::let_and_return)]
        let ptr = std::alloc::alloc(layout) as _;

        #[cfg(feature = "gc_trace")]
        eprintln!("allocating {:?}", ptr);

        ptr
    }

    /// Deallocate the memory used by a GC pointer.
    ///
    /// # Safety
    ///
    /// The pointer must not be reachable. Good luck!
    pub(crate) unsafe fn deallocate(&mut self, gc: GcAny) {
        #[cfg(feature = "gc_trace")]
        eprintln!("deallocating {:?}: {:?}", gc, gc.deref());

        let layout =
            Layout::from_size_align_unchecked(gc.deref().size(), Object::ALIGN);
        let ptr = std::mem::transmute(gc);
        std::alloc::dealloc(ptr, layout)
    }
}
