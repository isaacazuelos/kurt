//! A module is the result of compiling some code
//!
//! It's ready to be loaded by the runtime.
//!

use crate::{constant::Constant, prototype::Prototype};

#[derive(Debug, Clone, Default)]
pub struct Module {
    pub(crate) constants: Vec<Constant>,
    pub(crate) prototypes: Vec<Prototype>,
}

impl Module {
    /// Create a new empty module.
    pub fn new() -> Module {
        Module::default()
    }

    /// Get a reference to the module's constants.
    pub fn constants(&self) -> &[Constant] {
        self.constants.as_ref()
    }

    pub fn get_prototype(&self, index: usize) -> Option<&Prototype> {
        self.prototypes.get(index)
    }
}
