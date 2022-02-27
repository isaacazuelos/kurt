//! Index types.
//!
//! Instead of creating a bunch of different [newtype][0] structs for each type
//! of indexable thing, we instead use a [`Index`] struct for all of them which
//! takes a type parameter for the thing the index is for.
//!
//! This is better than just using `u32`s everywhere since we can have
//! associated constants, and the type checker will prevent us from trying to
//! get a prototype from a module by using a number that's meant to be an index
//! into some chunk of code.
//!
//! It also lets us have an overridable (but not dynamically dispatched)
//! [`get`][self::Indexable::get] method, which is neat if not especially
//! useful.
//!
//! Since it's a struct with `PhantomData`, we also know it won't take up any
//! extra space to do things this way.
//!
//! [0]: https://doc.rust-lang.org/rust-by-example/generics/new_types.html

use std::marker::PhantomData;

use crate::{constant::Constant, opcode::Op, prototype::Prototype};

/// An index which refers to a specific opcode within a
/// [`Code`][crate::code::Code] chunk.
///
/// This is a 'newtype' wrapper since we don't want people creating new
/// arbitrary indices or doing math on them.
#[derive()]
// NOTE: Since `derive` is a conditional impl on the generic parameters, we
//       can't really trust those to do the right thing -- since `T` doesn't
//       impact if we can compare/copy/etc. indexes.
pub struct Index<T>(pub(crate) u32, pub(crate) PhantomData<T>);

impl Index<Op> {
    /// The [`Index`] used to refer to the first opcode in some chunk of code.
    pub const START: Self = Index(0, PhantomData);

    /// The next index, returns `None` if it would overflow. This _is not_
    /// checking the underlying module to see if here's actually another module.
    pub fn next(self) -> Option<Self> {
        if self.0 == u32::MAX {
            None
        } else {
            Some(Index(self.0 + 1, PhantomData))
        }
    }
}

impl Index<Constant> {
    /// The largest constant index.
    pub const MAX: usize = u32::MAX as usize;
}

impl Index<Prototype> {
    /// The [`Index`] used to refer to a module's top-level code.
    pub const MAIN: Self = Index(0, PhantomData);
}

pub trait Indexable<T> {
    fn get(&self, index: Index<T>) -> Option<&T>;
}

impl<T> Clone for Index<T> {
    fn clone(&self) -> Self {
        Index(self.0, PhantomData)
    }
}

impl<T> Copy for Index<T> {}

impl<T> Eq for Index<T> {}

impl<T> PartialOrd for Index<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> PartialEq for Index<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> std::fmt::Debug for Index<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> Index<T> {
    /// Cast the [`Index`] into a [`usize`].
    #[inline(always)]
    pub fn as_usize(self) -> usize {
        self.0 as _
    }

    /// Cast the [`Index`] into a [`u32`].
    #[inline(always)]
    pub fn as_u32(self) -> u32 {
        self.0 as _
    }
}
