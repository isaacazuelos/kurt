//! An object is the result of compiling some code.
//!
//! It's ready for the runtime. Like a python `.pyc` or C `.o` file.

use common::Index;
use diagnostic::InputId;

use crate::{constant::Constant, internal::ModuleBuilder, Function};

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub(crate) input: Option<InputId>,
    pub(crate) constants: Vec<Constant>,
    pub(crate) functions: Vec<Function>,
}

impl Module {
    /// Index of the `main` function for this module, it's top-level code.
    pub const MAIN: Index<Function> = Index::START;

    /// The maximum number of functions that a module can contain.
    ///
    /// Note that top-level code in the module's `main` counts as one.
    pub const MAX_FUNCTIONS: usize = Index::<Function>::MAX;

    /// The maximum number of constants that a module can contain.
    pub const MAX_CONSTANTS: usize = Index::<Constant>::MAX;

    pub fn input(&self) -> Option<InputId> {
        self.input
    }

    pub fn constants(&self) -> &[Constant] {
        &self.constants
    }

    pub fn functions(&self) -> &[Function] {
        &self.functions
    }
}

impl std::ops::Index<Index<Function>> for Module {
    type Output = Function;

    fn index(&self, index: Index<Function>) -> &Self::Output {
        &self.functions[index.as_usize()]
    }
}

impl std::ops::Index<Index<Constant>> for Module {
    type Output = Constant;

    fn index(&self, index: Index<Constant>) -> &Self::Output {
        &self.constants[index.as_usize()]
    }
}

impl Default for Module {
    /// An empty module, the same thing you'd get from compiling an empty file.
    fn default() -> Self {
        Module::try_from("").unwrap()
    }
}

impl TryFrom<&str> for Module {
    type Error = diagnostic::Diagnostic;

    /// A helper for producing a whole (anonymous) module from just some input.
    fn try_from(input: &str) -> Result<Self, Self::Error> {
        Ok(ModuleBuilder::default().input(input)?.build())
    }
}
