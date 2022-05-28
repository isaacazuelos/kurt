//! An object is the result of compiling some code.
//!
//! It's ready for the runtime. Like a python `.pyc` or C `.o` file.

use common::{Get, Index};

use crate::{
    constant::Constant, debug::ModuleDebug, internal::ModuleBuilder, Function,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub(crate) constants: Vec<Constant>,
    pub(crate) functions: Vec<Function>,

    pub(crate) debug_info: Option<ModuleDebug>,
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

    /// The debug info for this module.
    pub fn debug_info(&self) -> Option<&ModuleDebug> {
        self.debug_info.as_ref()
    }

    /// Throw away the extra debug info this module carries.
    pub fn strip_debug(&mut self) {
        self.debug_info = None;
    }

    pub fn constants(&self) -> &[Constant] {
        &self.constants
    }

    pub fn functions(&self) -> &[Function] {
        &self.functions
    }
}

impl Get<Function> for Module {
    fn get(&self, index: Index<Function>) -> Option<&Function> {
        self.functions.get(index.as_usize())
    }
}

impl Get<Constant> for Module {
    fn get(&self, index: Index<Constant>) -> Option<&Constant> {
        self.constants.get(index.as_usize())
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
