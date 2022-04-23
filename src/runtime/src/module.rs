//! Runtime module representation.

use crate::value::Value;

use compiler::{
    constant::Constant,
    index::{Get, Index},
    prototype::Prototype,
};

#[derive(Debug, Default)]
pub struct Module {
    /// All the constants in this module.
    pub(crate) constants: Vec<Value>,

    /// The other prototypes used by this modules functions.
    pub(crate) prototypes: Vec<Prototype>,
}

impl Module {
    pub const MAIN_INDEX: Index<Prototype> = Index::new(0);
}

impl Get<Prototype> for Module {
    fn get(&self, index: Index<Prototype>) -> Option<&Prototype> {
        self.prototypes.get(index.as_usize())
    }
}

impl Get<Constant, Value> for Module {
    fn get(&self, index: Index<Constant>) -> Option<&Value> {
        self.constants.get(index.as_usize())
    }
}
