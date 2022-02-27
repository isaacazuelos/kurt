//! A module is the result of compiling some code
//!
//! It's ready to be loaded by the runtime. It's also where we'd serialize
//! modules, when the time comes.

use crate::{
    constant::Constant,
    index::{Index, Indexable},
    prototype::Prototype,
};

#[derive(Debug, Clone)]
pub struct Module {
    pub(crate) constants: Vec<Constant>,
    pub(crate) prototypes: Vec<Prototype>,
}

impl Default for Module {
    fn default() -> Self {
        Module::new()
    }
}

impl Module {
    /// Create a new empty module.
    pub fn new() -> Module {
        Module {
            constants: Vec::new(),
            prototypes: vec![Prototype::new_main()],
        }
    }
    /// Borrow the module's `main`, i.e. the top-level code.
    pub fn main(&self) -> &Prototype {
        &self.prototypes[0]
    }

    /// A view of all the constants in this module, the ordering matches the
    /// [`Index<Constant>`]s used within the module.
    pub fn constants(&self) -> &[Constant] {
        &self.constants
    }

    /// A view of all the constants in this module, the ordering matches the
    /// [`Index<Prototype>`]s used within the module.
    pub fn prototypes(&self) -> &[Prototype] {
        &self.prototypes
    }
}

impl Indexable<Prototype> for Module {
    fn get(&self, index: Index<Prototype>) -> Option<&Prototype> {
        self.prototypes.get(index.as_usize())
    }
}

impl Indexable<Constant> for Module {
    fn get(&self, index: Index<Constant>) -> Option<&Constant> {
        self.constants.get(index.as_usize())
    }
}
