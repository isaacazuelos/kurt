//! A module is the result of compiling some code
//!
//! It's ready to be loaded by the runtime.

use crate::{constant::Constant, prototype::Prototype};

#[derive(Debug, Clone)]
pub struct Module {
    pub(crate) constants: Vec<Constant>,
    pub(crate) prototypes: Vec<Prototype>,
    pub(crate) main: Prototype,
}

impl Module {
    /// Create a new empty module.
    pub fn new() -> Module {
        Module {
            constants: Vec::new(),
            prototypes: Vec::new(),
            main: Prototype::new_main(),
        }
    }

    /// Get a reference to the module's constants.
    pub fn constants(&self) -> &[Constant] {
        self.constants.as_ref()
    }

    /// Borrow the module's `main`, i.e. the top-level code.
    pub fn get_main(&self) -> &Prototype {
        &self.main
    }

    /// Get a prototype by it's index. Note that this does not include `main`.
    pub fn get_prototype(&self, index: usize) -> Option<&Prototype> {
        self.prototypes.get(index)
    }

    /// View the prototypes as a slice. Note that this does not include `main`.
    pub fn get_prototypes(&self) -> &[Prototype] {
        &self.prototypes
    }
}
