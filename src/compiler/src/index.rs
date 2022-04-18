//! Index types.
//!
//! Instead of creating a bunch of different [newtype][0] structs for each type
//! of indexable thing, we instead use a [`Index`] struct for all of them which
//! takes a type parameter for the thing the index is for.
//!
//! This is better than just using `u32`s everywhere since we can have
//! associated constants, and the type checker will prevent us from using
//! indexes in the wrong places.
//!
//! It also lets us have an overridable (but not dynamically dispatched)
//! [`get`][Get::get] method, which is neat.
//!
//! Since it's a struct with [`PhantomData`], we also know it won't take up any
//! extra space to do things this way.
//!
//! [0]: https://doc.rust-lang.org/rust-by-example/generics/new_types.html

use std::marker::PhantomData;

use crate::{constant::Constant, opcode::Op, prototype::Prototype};

/// An index which refers to a specific opcode.
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
    /// checking the underlying collection to see if here's actually another
    /// opcode.
    pub fn next(self) -> Option<Self> {
        if self.0 == u32::MAX {
            None
        } else {
            Some(Index(self.0 + 1, PhantomData))
        }
    }

    /// The previous index.
    ///
    /// However if the index is the start, it remains the start.
    pub fn pred_saturating(self) -> Self {
        Index(self.0.saturating_sub(1), PhantomData)
    }
}

impl Index<Constant> {
    /// The largest constant index.
    pub const MAX: usize = u32::MAX as usize;
}

impl Index<Prototype> {
    /// The [`Index`] used to refer to the top level or main code.
    pub const MAIN: Self = Index(0, PhantomData);
}

pub trait Get<In, Out = In> {
    fn get(&self, index: Index<In>) -> Option<&Out>;
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
    /// Create a new index from a u32.
    ///
    /// # Safety
    ///
    /// This mostly undoes the point of these [`Index`] types, but an escape
    /// hatch is useful. It's not `unsafe` since that feels a little too big a
    /// red flag for using this, but be careful!
    #[inline(always)]
    pub fn new(n: u32) -> Index<T> {
        Index(n as u32, PhantomData)
    }

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
