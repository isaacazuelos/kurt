//! Memory management

mod allocator;
mod any_managed;
mod collector;
mod gc;
mod string;
mod system_alloc;

use std::any::TypeId;

use self::collector::Trace;
pub use self::{any_managed::AnyManaged, gc::Gc};

/// Memory managed objects must conform to this trait to provide the system with
/// the information needed to safely manage them.
///
/// # Notes
///
/// Here is a [thread on the rust forums][forums] which describes what the
/// `'static` bound is doing here. This ensures that our managed types only use
/// owned (such as [`Box`]) or [`Gc`] pointers to values.
///
/// [forums]: https://users.rust-lang.org/t/37384
pub trait Managed: 'static + Trace {}

#[derive(Debug, Copy, Clone)]
pub struct Header {
    type_id: TypeId,
    colour: Tricolour,
}

impl Header {
    /// Create a new header.
    pub(crate) fn new<T: Managed + 'static>() -> Header {
        Header {
            type_id: TypeId::of::<T>(),
            colour: Tricolour::Black,
        }
    }

    /// Get the type tag, which is a [`TypeId`] so that the compiler manages
    /// generating them for us.
    pub(crate) fn tag(&self) -> TypeId {
        self.type_id
    }

    /// The current [`Tricolour`] of this object, which is used by the garbage
    /// collector.
    pub(crate) fn colour(&self) -> Tricolour {
        self.colour
    }

    /// Set the current GC [`Tricolour`].
    ///
    /// # Safety
    ///
    /// The collector should be the only thing which touches the colour. There
    /// are all sorts of invariants which must be maintained, depending on which
    /// GC strategy is used.
    pub(crate) unsafe fn set_colour(&mut self, colour: Tricolour) {
        self.colour = colour;
    }
}

/// Tricolour marking colours.
///
/// See [wikipedia][wiki] for more.
///
/// [wiki]: https://en.wikipedia.org/wiki/Tracing_garbage_collection#Tri-color_marking
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Tricolour {
    /// White objects have not yet been reached. They will be collected by teh
    /// next sweeping phase.
    White,
    /// Gray objects are those which are currently being traced -- they're known
    /// to be reachable, but not all of their children have added to the work
    /// list.
    Gray,
    /// Black objects are known reachable and have finished being processed.
    Black,
}
