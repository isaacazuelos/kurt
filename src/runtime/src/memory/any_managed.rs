//! Our [`Managed`] system's [top type][top].
//!
//! The idea is that a pointer to any concrete type which implements [`Managed`]
//! can be transmuted into a pointer to [`AnyManaged`] to safely access it's
//! header info or store it in makeshift heterogeneous collections (behind
//! pointers).
//!
//! There's probably a more elegant version of this with the
//! [`coerce_unsized`][std::ops::CoerceUnsized] feature, but I'm not certain and
//! not about to figure it out while it's unstable.
//!
//! [top]: https://en.wikipedia.org/wiki/Top_type

use super::{Header, Managed};

/// A value representing any [`Managed`] type.
///
/// You typically wouldn't want to create one of these directly, they're usually
/// made by [`Gc::upcast`][crate::memory::gc::Gc::upcast].
#[derive(Debug)]
#[repr(C, align(8))]
pub struct AnyManaged {
    header: Header,
}

unsafe impl Managed for AnyManaged {}

impl AnyManaged {
    /// Create a new managed value with no other properties.
    ///
    /// This isn't particularity useful.
    #[allow(unused)]
    fn new() -> AnyManaged {
        AnyManaged {
            header: Header::new::<AnyManaged>(),
        }
    }

    /// Get a copy of the [`Header`] for this value.
    ///
    /// Note that the header returned will contain the type tag with the actual
    /// [`TypeID`][std::any::TypeId] of the _actual_ type.
    ///
    /// We don't have inheritance, other than this one top type.
    pub fn header(&self) -> Header {
        self.header
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use super::AnyManaged;

    #[test]
    fn type_ids() {
        let a = AnyManaged::new();
        assert_eq!(a.header().tag(), a.type_id());
    }
}
