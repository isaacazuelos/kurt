//! Memory management

mod any_managed;
mod gc;

use std::any::TypeId;

pub use self::{any_managed::AnyManaged, gc::Gc};

/// A trait which indicates a type can be managed by our memory management
/// system.
///
/// # Notes
///
/// Here is a [thread on the rust forums][forums] which describes what the
/// `'static` bound is doing here. This ensures that our managed types only use
/// owned (such as [`Box`]) or [`Gc`] pointers to values.
///
/// [forums]: https://users.rust-lang.org/t/37384
///
/// # Safety
///
/// Any implementor of this trait must be annotated with `#[repr(C, align(8))]`
/// and have a [`Header`] as the first field. This allows the allocators and
/// collectors to safely cast to [`AnyManaged`] and access the value's
/// [`Header`].
pub unsafe trait Managed: 'static {
    const ALIGN: usize = 8;
}

#[derive(Debug, Copy, Clone)]
pub struct Header {
    type_id: TypeId,
}

impl Header {
    /// Create a new header.
    pub(crate) fn new<T: Managed + 'static>() -> Header {
        Header {
            type_id: TypeId::of::<T>(),
        }
    }

    /// Get the type tag, which is a [`TypeId`] so that the compiler manages
    /// generating them for us.
    pub(crate) fn tag(&self) -> TypeId {
        self.type_id
    }
}
