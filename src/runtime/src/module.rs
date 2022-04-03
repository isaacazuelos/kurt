//! Runtime module representation.

use crate::{
    error::{Error, Result},
    value::Value,
};

use compiler::{constant::Constant, index::Index, prototype::Prototype};

#[derive(Debug)]
pub struct Module {
    pub(crate) main: Prototype,

    /// All the constants in this module.
    pub(crate) constants: Vec<Value>,

    /// The other prototypes used by this modules functions.
    pub(crate) prototypes: Vec<Prototype>,
}

impl Default for Module {
    fn default() -> Self {
        Module {
            main: Prototype::new_main(),
            prototypes: Vec::new(),
            constants: Vec::new(),
        }
    }
}

impl Module {
    /// Look up a constant by an [`Index`].
    pub(crate) fn constant(&self, index: Index<Constant>) -> Result<Value> {
        self.constants
            .get(index.as_usize())
            .cloned()
            .ok_or(Error::ConstantIndexOutOfRange)
    }

    /// Look up a prototype by an [`Index`].
    pub(crate) fn prototype(
        &self,
        index: Index<Prototype>,
    ) -> Result<&Prototype> {
        if index == Index::MAIN {
            Ok(&self.main)
        } else {
            self.prototypes
                .get(index.as_usize() - 1)
                .ok_or(Error::PrototypeIndexOutOfRange)
        }
    }
}
