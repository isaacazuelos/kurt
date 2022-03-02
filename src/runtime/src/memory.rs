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

use std::alloc::Layout;

use crate::Runtime;

#[derive(Debug, Clone)]
pub struct Gc<T: Managed>(*mut T);

/// Any value which can be managed by the garbage collector.
pub trait Managed {}

/// Since our `Managed` values can be [DSTs][dst], we need a way to initialize
/// them. This trait helps us do that.
///
/// [dst]: https://doc.rust-lang.org/nomicon/exotic-sizes.html
pub(crate) trait InitFrom<A> {
    /// Returns the [`Layout`] needed for a value of our type when initialized
    /// from the given argument.
    fn layout(arg: &A) -> Layout;

    /// Initialize the pointer using the given value.
    ///
    /// # Safety
    ///
    /// 1. The pointer must satisfy the layout given by [`InitFrom::layout`] fo
    ///    the same arguments.
    ///
    /// 2. The pointer must be to uninitialized memory.
    ///
    /// 3. The pointer must be initialized when the function returns.
    unsafe fn init(ptr: *mut Self, args: A);
}

impl<T, A> InitFrom<A> for T
where
    T: From<A> + Sized,
{
    fn layout(_arg: &A) -> Layout {
        Layout::new::<Self>()
    }

    unsafe fn init(ptr: *mut Self, arg: A) {
        ptr.write(Self::from(arg))
    }
}

#[allow(dead_code)]
impl Runtime {
    /// Make a new [`Managed`] object using it's [`Default`] instance.
    pub(crate) fn make_default<T>(&mut self) -> Gc<T>
    where
        T: Managed + Default,
    {
        self.make(T::default())
    }

    /// Make a new [`Managed`] object, initializing it from the given argument.
    pub(crate) fn make<T, A>(&mut self, arg: A) -> Gc<T>
    where
        T: Managed + InitFrom<A>,
    {
        let layout = T::layout(&arg);
        unsafe {
            let raw = self.allocate(layout) as *mut T;
            T::init(raw, arg);
            Gc(raw)
        }
    }

    /// Allocate for the given layout.
    fn allocate(&mut self, layout: Layout) -> *mut u8 {
        // Just leaks for now.
        unsafe { std::alloc::alloc(layout) }
    }
}
