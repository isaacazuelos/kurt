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

use crate::memory::{
    collector::{Trace, WorkList},
    Managed,
};

/// A value representing any [`Managed`] type.
///
/// You typically wouldn't want to create one of these directly, they're usually
/// made by [`Gc::upcast`][crate::memory::gc::Gc::upcast].
pub struct AnyManaged;

impl Managed for AnyManaged {}

impl Trace for AnyManaged {
    fn trace(&mut self, _worklist: &mut WorkList) {
        todo!("How do we downcast dynamically to trace?")
    }
}
