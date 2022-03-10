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
mod gc;
mod object;

pub mod string;

use crate::{memory::class::Class, Runtime};

pub use self::gc::GcObj;

pub(crate) use self::object::Object;

/// Since our [`Class`] values can be [DSTs][dst], we need a way to initialize
/// them. This trait helps us do that.
///
/// [dst]: https://doc.rust-lang.org/nomicon/exotic-sizes.html
pub(crate) trait InitFrom<A> {
    /// Returns the size the [`Class`] needs in bytes, when initialized from the
    /// given argument.
    ///
    /// Note that this isn't the total size of the object, just the size of the
    /// [`Class`] itself.
    fn size(arg: &A) -> usize;

    /// Initialize the pointer using the given value.
    ///
    /// # Safety
    ///
    /// 1. The pointer must not be null.
    ///
    /// 1. The pointer must be to uninitialized memory.
    ///
    /// 1. The pointer must point to an allocation large enough, as given by
    ///    [`InitFrom::size`] for the given argument.
    ///
    /// When the function returns, the pointer should point to an initialized
    /// instance of `C`.
    unsafe fn init(ptr: *mut Self, args: A);
}

impl<T, A> InitFrom<A> for T
where
    T: From<A> + Sized,
{
    fn size(_arg: &A) -> usize {
        std::mem::size_of::<Self>()
    }

    unsafe fn init(ptr: *mut Self, arg: A) {
        ptr.write(Self::from(arg))
    }
}

impl Runtime {
    /// Make a new [`Managed`] object using it's [`Default`] instance.
    #[allow(dead_code)]
    pub(crate) fn make<C>(&mut self) -> GcObj
    where
        C: Class + Default,
    {
        self.make_from::<C, _>(C::default())
    }

    /// Make a new [`Object`], initializing it from the given argument.
    pub(crate) fn make_from<C, A>(&mut self, arg: A) -> GcObj
    where
        C: Class + InitFrom<A>,
    {
        // find the layout needed for the object.
        let extra = C::size(&arg);
        let layout = Self::object_layout_with_extra(extra);

        // SAFETY: We're leaking by design, for now.
        let raw = unsafe { self.allocate(layout) };

        // Initialize the object. This is first done to the base `Object`, then
        // to the specific `Class`. Kind of like a super.init() call before
        // self.init() in a more normal OO system.
        unsafe {
            Object::init(raw, C::TAG, layout.size());
            C::init(raw as *mut C, arg);
        };

        // Crate the GC pointer.
        unsafe { GcObj::from_non_null(NonNull::new_unchecked(raw)) }
    }

    fn object_layout_with_extra(extra: usize) -> Layout {
        const ALIGN: usize = std::mem::align_of::<Object>();
        const BASE_SIZE: usize = std::mem::size_of::<Object>();

        // SAFETY: Align is explicitly set for Object to 8, so it's non-zero
        //        and a power of 2. We don't check for overflows on `size`
        //        though -- should we?
        unsafe { Layout::from_size_align_unchecked(BASE_SIZE + extra, ALIGN) }
    }

    /// Allocate for the given layout.
    ///
    /// # Safety
    ///
    /// This memory isn't tracked in any way, so it will leak. The caller is
    /// responsible for ensuring it is freed appropriately.
    ///
    /// The pointer returned satisfies the layout, is not null, but is not
    /// initialized either.
    unsafe fn allocate(&mut self, layout: Layout) -> *mut Object {
        // Just leaks for now.
        std::alloc::alloc(layout) as _
    }
}
