//! Compiler - Turns syntax trees into modules the runtime can load.
//!
//! Each module is compiled completely independently, and they're linked
//! together by the runtime when loaded.
//!
//! You can think of the [`Compiler`] struct as being the context that's built
//! up as you add code to the module with [`Compiler::compile`].
//!
//! Once you're ready to produce a [`Module`], you can do so by calling
//! [`Compiler::build`].

use diagnostic::Span;
use syntax::ast;

use crate::{
    constant::Pool, error::Result, opcode::Op, prototype::Prototype, Module,
};

mod rules;

/// A compiler turns source code into a module the runtime can work with. It
/// keeps track of all the state used when compiling a module.
#[derive(Clone)]
pub struct Compiler {
    /// The constant pool of all constants seen by this compiler so far.
    constants: Pool,

    /// Code is compiled into [`Prototype`]s which are kept as a stack that
    /// matches closure scopes in the source code. This [`Vec`] should never be
    /// empty, as the first element is the top-level's prototype.
    prototypes: Vec<Prototype>,
}

impl Compiler {
    /// Create a new compiler.
    ///
    /// If you're just creating a module in one go, you probably want to use the
    /// top-level [`compile`][crate::compile] function.
    ///
    /// # Example
    ///
    /// ```
    /// # use compiler::Compiler;
    /// use syntax::ast::{self, *};
    ///
    /// let mut compiler = Compiler::new();
    ///
    /// // You can push code to the module the compiler is building.
    /// let code = ast::Module::parse(r#" "Hello, world!"; "#).unwrap();
    /// compiler.compile(&code).unwrap();
    ///
    /// // Once you're ready to build a module the runtime can work with you
    /// // need to call `build`.
    /// let module = compiler.build().unwrap();
    ///
    /// // Now you can do things with the module.
    /// ```
    #[allow(clippy::new_without_default)]
    pub fn new() -> Compiler {
        let prototypes = vec![Prototype::new_top_level()];

        Compiler {
            prototypes,
            constants: Pool::default(),
        }
    }

    /// Push some code through the compiler (and in a sense on to the end of the
    /// currently-compiling module).
    ///
    /// The input must be valid top-level code (which is the same as what's
    /// allowed in a module for now).
    ///
    /// # Examples
    ///
    /// See the documentation for [`Compiler::new`] for an example of how this
    /// can be used.
    pub fn compile(&mut self, syntax: &ast::Module) -> Result<()> {
        // This is on hold as I'm not really sure how to handle pushing code
        // that produces an error.
        //
        // - Is the compiler just in a potentially invalid state after, and
        //   caller have to [`Clone`] first?
        // - Do we have some sort of 'waypoint'/'backtrack' system to backtrack?
        // - How far back can we go -- maybe each top-level item is reversible,
        //   that way we only need to 'backtrack' in the top_level prototype?
        //
        // How do we handle pushing code that's incomplete -- say ends in the
        // middle of a closure or something?
        self.module(syntax)
    }

    /// Convert the current compiler state into a new module that can be loaded
    /// into the runtime.
    ///
    /// # Examples
    ///
    /// See the documentation for [`Compiler::new`] for an example of how this
    /// can be used.
    pub fn build(&self) -> Result<Module> {
        Ok(Module {
            constants: self.constants.as_vec(),
            prototypes: self.prototypes.clone(),
        })
    }
}

// Local Helpers
impl Compiler {
    /// This is just a shorthand for emitting to the current active prototype.
    pub(crate) fn emit(&mut self, op: Op, span: Span) -> Result<()> {
        self.active_prototype_mut().emit(op, span)
    }

    /// Get a mutable reference to the active prototype. This will return the
    /// top level prototype if we're not compiling into a closure.
    pub(crate) fn active_prototype_mut(&mut self) -> &mut Prototype {
        self.prototypes
            .last_mut()
            .expect("Compiler.prototypes should never be empty.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Not a lot of tests as there's not really much happening right now.

    #[test]
    fn active_prototype() {
        let mut c = Compiler::new();
        assert!(!c.prototypes.is_empty()); // since 0 should be the top-level
        let _ = c.active_prototype_mut(); // shouldn't panic
    }
}
