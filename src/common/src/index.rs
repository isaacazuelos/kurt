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

use std::{
    fmt::{Display, Formatter},
    marker::PhantomData,
};

/// A 32-bit index.
///
/// This is a 'newtype' wrapper since we don't want people creating new
/// arbitrary indices or carelessly doing math on them.
#[derive()]
// NOTE: Since `derive` is a conditional impl on the generic parameters, we
//       can't really trust those to do the right thing -- since `T` doesn't
//       impact if we can compare/copy/etc. indexes.
pub struct Index<T>(u32, PhantomData<T>);

impl<T> Index<T> {
    /// Index 0, the starting index.
    pub const START: Index<T> = Index(0, PhantomData);

    /// The largest any [`Index`] can be.
    pub const MAX: usize = u32::MAX as usize;

    /// The largest [`Index`].
    pub const MAX_INDEX: Index<T> = Index::new(u32::MAX);

    /// Create a new index from a u32.
    ///
    /// # Safety
    ///
    /// While now marked `unsafe`, this mostly undoes the point. Try not to use
    /// this if you're not the one consuming the index later.
    #[inline(always)]
    pub const fn new(n: u32) -> Index<T> {
        Index(n as u32, PhantomData)
    }

    /// Cast the [`Index`] into a [`usize`].
    #[inline(always)]
    pub fn as_usize(self) -> usize {
        self.0 as _
    }

    /// The next index, returning `None` if it would overflow.
    pub fn next(self) -> Option<Self> {
        let n = self.0.checked_add(1)?;
        Some(Index::new(n))
    }

    /// The next index, returning the same value if it would overflow.
    pub fn saturating_next(self) -> Self {
        Index(self.0.saturating_add(1), PhantomData)
    }

    /// The previous index, returning `None` if it would underflow.
    pub fn previous(self) -> Option<Index<T>> {
        let n = self.0.checked_sub(1)?;
        Some(Index::new(n))
    }

    /// The previous index, returning the same value if it would underflow.
    pub fn saturating_previous(self) -> Self {
        Index(self.0.saturating_sub(1), PhantomData)
    }

    /// Increment the index in-place if doing so does not overflow.
    pub fn saturating_increment(&mut self) {
        *self = self.saturating_next();
    }

    /// Decrement the index in-place if doing so does not underflow.
    pub fn saturating_decrement(&mut self) {
        *self = self.saturating_previous();
    }
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

impl<T> Display for Index<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl<T> From<Index<T>> for u32 {
    fn from(n: Index<T>) -> Self {
        n.0
    }
}

impl<T> From<Index<T>> for usize {
    fn from(n: Index<T>) -> Self {
        n.0 as usize
    }
}
