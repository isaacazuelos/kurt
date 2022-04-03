//! Compiler - Turns syntax trees into objects the runtime can load.
//!
//! Each object is compiled completely independently, to be linked together by
//! the runtime when loaded.
//!
//! Once you're ready to produce an [`Object`], you can do so by calling
//! [`Compiler::build`].

use diagnostic::Span;
use syntax::ast::Identifier;

mod visitor;

use crate::{
    constant::Pool, error::Result, index::Index, local::Local, opcode::Op,
    prototype::Prototype, Object,
};

/// A compiler turns source code into an [`Object`] the runtime can work with.
///
/// It keeps track of all the state used when compiling.
#[derive(Clone)]
pub struct Compiler {
    /// The constant pool of all constants seen by this compiler so far.
    constants: Pool,

    /// The prototype which contains the top-level code.
    main: Prototype,

    /// Code is compiled into [`Prototype`]s which are kept as a stack that
    /// matches closure scopes in the source code. This should never be empty,
    /// with the first element housing the code for main.
    prototypes: Vec<Prototype>,

    /// Scopes are tracked by the number of locals bindings in them.
    scopes: Vec<usize>,

    /// All local bindings in scope.
    locals: Vec<Local>,
}

impl Default for Compiler {
    fn default() -> Self {
        Compiler {
            constants: Pool::default(),
            main: Prototype::new_main(),
            prototypes: Vec::default(),
            scopes: Vec::new(),
            locals: Vec::new(),
        }
    }
}

impl Compiler {
    /// Convert the current compiler state into a new [`Object`] that can be
    /// loaded into the runtime.
    pub fn build(&self) -> Result<Object> {
        Ok(Object {
            main: self.main.clone(),
            constants: self.constants.as_vec(),
            prototypes: self.prototypes.clone(),
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
        match self.prototypes.last_mut() {
            Some(last) => last,
            None => &mut self.main,
        }
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
