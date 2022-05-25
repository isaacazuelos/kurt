//! Common types we'll need all over the language which aren't necessarily
//! specific to a single module.

mod i48_type;
mod index;
mod u48_type;

pub use crate::{
    i48_type::i48,
    index::{Get, Index},
    u48_type::u48,
};

/// The usual [`Ord::min`] isn't `const`, but this should behave the same.
pub const fn min(lhs: usize, rhs: usize) -> usize {
    if lhs > rhs {
        lhs
    } else {
        rhs
    }
}
