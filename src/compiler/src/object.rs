//! An object is the result of compiling some code.
//!
//! It's ready for the runtime. Like a python `.pyc` or C `.o` file.

use crate::{
    constant::Constant,
    index::{Index, Indexable},
    prototype::Prototype,
};

#[derive(Debug, Clone)]
pub struct Object {
    pub(crate) constants: Vec<Constant>,
    pub(crate) main: Prototype,
    pub(crate) prototypes: Vec<Prototype>,
}

impl Default for Object {
    fn default() -> Self {
        Object {
            constants: Vec::new(),
            main: Prototype::new_main(),
            prototypes: Vec::new(),
        }
    }
}

impl Object {
    /// The prototype containing top-level code.
    pub fn main(&self) -> &Prototype {
        &self.main
    }

    /// A view of all the constants used, the ordering matches the
    /// [`Index<Constant>`]s used within prototypes.
    pub fn constants(&self) -> &[Constant] {
        &self.constants
    }

    /// A view of all the constants used, the indexes match the
    /// [`Index<Prototype>`]s used by code within the object.
    pub fn prototypes(&self) -> &[Prototype] {
        &self.prototypes
    }
}

impl Indexable<Prototype> for Object {
    fn get(&self, index: Index<Prototype>) -> Option<&Prototype> {
        self.prototypes.get(index.as_usize())
    }
}

impl Indexable<Constant> for Object {
    fn get(&self, index: Index<Constant>) -> Option<&Constant> {
        self.constants.get(index.as_usize())
    }
}
