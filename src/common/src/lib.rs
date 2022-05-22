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
