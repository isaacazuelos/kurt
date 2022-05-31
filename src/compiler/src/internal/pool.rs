//! A constant pool is where the compiler stores all the constants it'll need
//! for a module while compiling.

use std::collections::HashMap;

use common::{Get, Index};

use crate::Constant;

/// A pool of constants, indexable by an [`Index`]. Inserting a constant returns
/// the index, and if the constant is already in the pool its existing index is
/// reused.
#[derive(Default, Clone)]
pub struct ConstantPool {
    constants: HashMap<Constant, Index<Constant>>,
}

impl ConstantPool {
    /// The maximum number of constant values we can store in a pool.
    pub const MAX_CONSTANTS: usize = u32::MAX as usize;

    /// The number of unique constants in the pool.
    pub fn len(&self) -> usize {
        self.constants.len()
    }

    /// Insert a constant into a this pool. If it's already present, the
    /// existing [`Index`] is returned, otherwise a new one is used.
    pub fn insert(
        &mut self,
        constant: impl Into<Constant>,
    ) -> Option<Index<Constant>> {
        let len = self.constants.len();

        if len > ConstantPool::MAX_CONSTANTS {
            None
        } else {
            Some(
                *self
                    .constants
                    .entry(constant.into())
                    .or_insert_with(|| Index::new(len as u32)),
            )
        }
    }

    /// Turn this into a [`Vec<Constant>`] where each constant's position in teh
    /// array is it's [Index].
    pub(crate) fn as_vec(&self) -> Vec<Constant> {
        let mut vec: Vec<_> = std::iter::repeat(Constant::Float(0))
            .take(self.constants.len())
            .collect();

        for (k, v) in &self.constants {
            vec[v.as_usize()] = k.clone();
        }

        vec
    }

    /// Keep the constants with an index less than `len`, which mean keeping the
    /// constants until [`Pool::len`] is `len`.
    pub(crate) fn truncate(&mut self, len: usize) {
        self.constants.retain(|_, i| i.as_usize() < len)
    }
}

impl Get<Constant> for ConstantPool {
    fn get(&self, index: Index<Constant>) -> Option<&Constant> {
        for k in self.constants.keys() {
            if let Some(c) = self.constants.get(k) {
                if index == *c {
                    return Some(k);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_new() {
        let mut pool = ConstantPool::default();
        let i1 = pool.insert(Constant::Float(0)).unwrap();
        let i2 = pool.insert(Constant::Float(1)).unwrap();
        assert_ne!(i1, i2);
    }

    #[test]
    fn insert_dup() {
        let mut pool = ConstantPool::default();
        let i1 = pool.insert(Constant::Float(0)).unwrap();
        let i2 = pool.insert(Constant::Float(0)).unwrap();
        assert_eq!(i1, i2);
    }

    #[test]
    fn into_vec() {
        let mut pool = ConstantPool::default();
        pool.insert(Constant::Float(6)).unwrap();
        pool.insert(Constant::Float(5)).unwrap();
        assert_eq!(pool.as_vec(), [Constant::Float(6), Constant::Float(5)]);
    }
}
