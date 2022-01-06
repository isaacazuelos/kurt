//! Memory management

mod allocator;
mod any_managed;
mod gc;
mod string;
mod system_alloc;

use std::any::TypeId;

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
///
/// # Safety
///
/// This can only be implemented by a struct, and we need a few extra
/// guarantees.
///
/// 1. The struct must be `#[repr(C, align(16))]`. This means all objects have
///    the same alignment and our fields are laid out in the order we specify.
///    We use `align(16)` because that's what intel recommends for structures
///    larger than 64-bits.
///
/// 2. The first field of the struct must be a [`Header`][crate::gc::Header].
///    This is what allows us to go between [`Gc<T>`][crate::gc::Gc] and
///    [`GcRaw`][crate::gc::GcRaw] safely.
pub unsafe trait Managed: 'static {
    const ALIGN: usize = 16;
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
