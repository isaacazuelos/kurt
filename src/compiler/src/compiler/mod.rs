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
use syntax::ast::{self, Identifier};

mod visitor;

use crate::{
    constant::Pool, error::Result, index::Index, local::Local, opcode::Op,
    prototype::Prototype, Module,
};

/// A compiler turns source code into a module the runtime can work with. It
/// keeps track of all the state used when compiling a module.
#[derive(Clone)]
pub struct Compiler {
    /// The constant pool of all constants seen by this compiler so far.
    constants: Pool,

    /// Code is compiled into [`Prototype`]s which are kept as a stack that
    /// matches closure scopes in the source code. This should never be empty,
    /// with the first element being the module's top-level code.
    prototypes: Vec<Prototype>,

    /// Scopes are tracked by the number of locals bindings in them.
    scopes: Vec<usize>,

    /// All local bindings in scope.
    locals: Vec<Local>,
}

impl Default for Compiler {
    fn default() -> Self {
        Compiler::new()
    }
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
    ///
    /// // Once you're ready to build a module the runtime can work with you
    /// // need to call `build`.
    /// let module = compiler.compile(&code).unwrap().build();
    ///
    /// // Now you can do things with the module.
    /// ```
    pub fn new() -> Compiler {
        Compiler {
            prototypes: vec![Prototype::new_main()],
            constants: Pool::default(),

            scopes: vec![0],
            locals: Vec::new(),
        }
    }

    /// Push some code through the compiler (and in a sense on to the end of the
    /// currently-compiling module). This consumes the compiler, returning it
    /// only if the new code can safely be compiled. This lets us avoid trying
    /// to backtrack the compiler state to some previously known-good state.
    ///
    /// One consequence of this is if an input source like the reply tries to
    /// pass two statements at once, even if the first succeeds in compiling,
    /// the input overall will not.
    ///
    /// Whereas  python `print('hi'); unbound` will print 'hi' before producing
    /// a NameError, our language will not.
    ///
    /// The input must be valid top-level code (which is the same as what's
    /// allowed in a module for now).
    ///
    /// Note that this consumes the compiler, returning it only if the new
    /// syntax can be compiled. If you need that old compiler state, [`Clone`]
    /// the compiler.
    ///
    /// # Examples
    ///
    /// See the documentation for [`Compiler::new`] for an example of how this
    /// can be used.
    pub fn compile(mut self, syntax: &ast::Module) -> Result<Self> {
        // Suppose we called `build` on this _before_ this call to compile, and
        // run that module.
        //
        // The call to build injected a `Op::Halt` that the program counter is
        // on right now. We need to account for that, so we'll put in a no-op to
        // keep the program counter aligned before the next _real_ instruction.
        //
        // If the module is being restarted, the stack has either zero or one
        // values on it.
        //
        // The stack may be empty due to an empty module or a trailing ';' to
        // suppress the last value, or it may have the one un-suppressed last
        // statement's value on it.
        //
        // We don't need to worry about which, since [`Op::Pop`] is a no-op on
        // an empty stack.
        if !self.prototypes[0].is_empty() {
            self.prototypes[0].emit_synthetic(Op::Nop)?;
            self.prototypes[0].emit_synthetic(Op::Pop)?;
        }

        // So we're good to just keep compiling statements.
        self.module(syntax)?;
        Ok(self)
    }

    /// Convert the current compiler state into a new module that can be loaded
    /// into the runtime.
    ///
    /// # Examples
    ///
    /// See the documentation for [`Compiler::new`] for an example of how this
    /// can be used.
    pub fn build(&self) -> Result<Module> {
        let mut prototypes = self.prototypes.clone();

        prototypes[0].emit_synthetic(Op::Halt)?;

        Ok(Module {
            constants: self.constants.as_vec(),
            prototypes,
        })
    }
}

// Helpers used by the visitors
impl Compiler {
    /// This is just a shorthand for emitting to the current active prototype.
    pub(crate) fn emit(&mut self, op: Op, span: Span) -> Result<()> {
        self.active_prototype_mut().emit(op, span)
    }

    /// Get a mutable reference to the active prototype. This will return the
    /// prototype used by `main` if we're not compiling a closure.
    pub(crate) fn active_prototype_mut(&mut self) -> &mut Prototype {
        // This is safe as a all modules have at least one prototype -- 'main',
        // the top-level code.
        self.prototypes.last_mut().unwrap()
    }
}

// Bindings and scopes
impl Compiler {
    pub(crate) fn _begin_scope(&mut self) {
        self.scopes.push(0);
    }

    pub(crate) fn _end_scope(&mut self) {
        let going_out_of_scope_count =
            self.scopes.pop().expect("scopes cannot be empty");

        self.locals
            .truncate(self.locals.len() - going_out_of_scope_count);
    }

    pub(crate) fn bind_local(&mut self, id: &Identifier) {
        // Do we have space for another local in this scope?
        let current_scope_count =
            self.scopes.last().cloned().unwrap_or_default();

        if current_scope_count >= u32::MAX as usize {
            panic!(
                "cannot have more than {} (i.e. u32::MAX) locals.",
                u32::MAX
            );
        }

        // Okay, we can actually bind it.
        let local = Local::from(id);
        self.locals.push(local);

        // We need to update our scope tracking too.
        if let Some(current_scope_count) = self.scopes.last_mut() {
            *current_scope_count += 1;
        }
    }

    pub(crate) fn resolve_local(&mut self, name: &str) -> Option<Index<Local>> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.as_str() == name {
                return Some(Index::new(i as _));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_prototype() {
        let mut c = Compiler::new();
        assert_eq!(c.prototypes.len(), 1);
        // active_prototype is main when prototypes is empty, casting is to do
        // pointer equality (i.e. identity).
        assert_eq!(
            c.active_prototype_mut() as *mut _,
            &mut c.prototypes[0] as *mut _
        );
    }
}
