//! Compiler - Turns syntax trees into objects the runtime can load.
//!
//! Each object is compiled completely independently, to be linked together by
//! the runtime when loaded.
//!
//! Once you're ready to produce an [`Object`], you can do so by calling
//! [`Compiler::build`].

use diagnostic::Span;
use syntax::{Identifier, Syntax};

mod visitor;

use crate::{
    constant::Pool, error::Error, error::Result, index::Index, local::Local,
    opcode::Op, prototype::Prototype, Object,
};

/// A compiler turns source code into an [`Object`] the runtime can work with.
///
/// It keeps track of all the state used when compiling.
#[derive(Clone, Default)]
pub struct Compiler {
    /// The constant pool of all constants seen by this compiler so far.
    constants: Pool,

    /// A stack of currently compiling prototypes. Once completed, they're moved
    /// to `prototypes`.
    compiling: Vec<Prototype>,

    /// Code is compiled into [`Prototype`]s which are kept here once complete
    prototypes: Vec<Prototype>,
}

impl Compiler {
    /// Convert the current compiler state into a new [`Object`] that can be
    /// loaded into the runtime.
    pub fn build(&self) -> Result<Object> {
        // TODO: we could go back to top-level code being index 0 here.
        let mut prototypes = self.prototypes.clone();
        let main = self.compiling.last().unwrap().clone();
        prototypes.push(main);

        Ok(Object {
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
        self.compiling.last_mut().unwrap()
    }
}

// Bindings and scopes
impl Compiler {
    pub(crate) fn with_scope<F, T>(&mut self, inner: F) -> Result<T>
    where
        F: FnOnce(&mut Compiler) -> Result<T>,
    {
        self.active_prototype_mut().begin_scope();

        let result = inner(self);

        self.active_prototype_mut().end_scope();

        result
    }

    pub(crate) fn with_prototype<F>(
        &mut self,
        inner: F,
    ) -> Result<Index<Prototype>>
    where
        F: FnOnce(&mut Compiler) -> Result<()>,
    {
        if self.compiling.len() + self.prototypes.len() >= u32::MAX as usize {
            return Err(Error::TooManyPrototypes);
        }

        self.compiling.push(Prototype::new());

        inner(self)?;

        let prototype = self.compiling.pop().unwrap();
        self.prototypes.push(prototype);

        Ok(Index::new((self.prototypes.len() - 1) as u32))
    }

    pub(crate) fn bind_local(&mut self, id: &Identifier) {
        self.active_prototype_mut()
            .bind_local(Local::new(id.as_str(), id.span()))
    }

    pub(crate) fn resolve_local(&mut self, name: &str) -> Option<Index<Local>> {
        self.active_prototype_mut().resolve_local(name)
    }
}
