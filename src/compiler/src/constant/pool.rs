//! A constant pool is where the compiler stores all the constants it'll need
//! for a module while compiling.

use std::{collections::HashMap, marker::PhantomData};

use crate::{
    constant::Constant,
    error::{Error, Result},
    index::Index,
};

/// A pool of constants in the module, indexable by an [`Index`]. Inserting a
/// constant returns an index, and if the constant is already in the pool its
/// existing index is reused.
#[derive(Default, Clone)]
pub struct Pool {
    constants: HashMap<Constant, Index<Constant>>,
}

impl Pool {
    /// Insert a constant into a this pool. If it's already present, the
    /// existing [`Index`] is returned, otherwise a new one is used.
    pub fn insert(
        &mut self,
        constant: impl Into<Constant>,
    ) -> Result<Index<Constant>> {
        let len = self.constants.len();

        if len > Index::<Constant>::MAX {
            Err(Error::TooManyConstants)
        } else {
            Ok(*self
                .constants
                .entry(constant.into())
                .or_insert_with(|| Index(len as u32, PhantomData)))
        }
    }

    /// Turn this into a [`Vec<Constant>`] where each constant's position in teh
    /// array is it's [Index].
    pub(crate) fn as_vec(&self) -> Vec<Constant> {
        let mut vec: Vec<_> = std::iter::repeat(Constant::Number(0))
            .take(self.constants.len())
            .collect();

        for (k, Index(v, _)) in &self.constants {
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
        let i1 = pool.insert(Constant::Number(0)).unwrap();
        let i2 = pool.insert(Constant::Number(1)).unwrap();
        assert_ne!(i1, i2);
    }

    #[test]
    fn insert_dup() {
        let mut pool = Pool::default();
        let i1 = pool.insert(Constant::Number(0)).unwrap();
        let i2 = pool.insert(Constant::Number(0)).unwrap();
        assert_eq!(i1, i2);
    }

    #[test]
    fn into_vec() {
        let mut pool = Pool::default();
        pool.insert(Constant::Number(6)).unwrap();
        pool.insert(Constant::Number(5)).unwrap();
        assert_eq!(pool.as_vec(), [Constant::Number(6), Constant::Number(5)]);
    }
}
