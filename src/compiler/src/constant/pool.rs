//! A constant pool is where the compiler stores all the constants it'll need
//! for a module while compiling.

use std::collections::HashMap;

use crate::{
    constant::Constant,
    error::{Error, Result},
};

/// An index into a [`ConstantPool`].
///
/// This is a 'newtype' wrapper since we don't want people creating new
/// arbitrary indices or doing math on them.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Index(u32);

impl Index {
    /// The maximum number of constants that a single module can refer to.
    pub const MAX: usize = u32::MAX as _;
}

/// A pool of constants in the module, indexable by an [`Index`]. Inserting a
/// constant returns an index, and if the constant is already in the pool its
/// existing index is reused.
#[derive(Default, Clone)]
pub struct Pool {
    constants: HashMap<Constant, Index>,
}

impl Pool {
    /// Insert a constant into a this pool. If it's already present, the
    /// existing [`Index`] is returned, otherwise a new one is used.
    pub fn insert(&mut self, constant: impl Into<Constant>) -> Result<Index> {
        let len = self.constants.len();

        if len > Index::MAX {
            Err(Error::TooManyConstants)
        } else {
            Ok(*self
                .constants
                .entry(constant.into())
                .or_insert_with(|| Index(len as u32)))
        }
    }

    /// Turn this into a [`Vec<Constant>`] where each constant's position in teh
    /// array is it's [Index].
    pub(crate) fn as_vec(&self) -> Vec<Constant> {
        let mut vec: Vec<_> = std::iter::repeat(Constant::Number(0))
            .take(self.constants.len())
            .collect();

        for (k, Index(v)) in &self.constants {
            vec[*v as usize] = k.clone();
        }

        vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_new() {
        let mut pool = Pool::default();
        let Index(i) = pool.insert(Constant::Number(0)).unwrap();
        assert_eq!(i, 0);
        let Index(j) = pool.insert(Constant::Number(1)).unwrap();
        assert_eq!(j, 1);
    }

    #[test]
    fn insert_dup() {
        let mut pool = Pool::default();
        let Index(i) = pool.insert(Constant::Number(0)).unwrap();
        let Index(j) = pool.insert(Constant::Number(0)).unwrap();
        assert_eq!(j, i);
    }

    #[test]
    fn into_vec() {
        let mut pool = Pool::default();
        pool.insert(Constant::Number(6)).unwrap();
        pool.insert(Constant::Number(5)).unwrap();
        assert_eq!(pool.as_vec(), [Constant::Number(6), Constant::Number(5)]);
    }
}
