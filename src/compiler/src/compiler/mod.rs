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
#[derive(Clone)]
pub struct Compiler {
    /// The constant pool of all constants seen by this compiler so far.
    constants: Pool,

    /// A stack of currently compiling prototypes. Once completed, they're moved
    /// to `prototypes`.
    compiling: Vec<Prototype>,

    /// Code is compiled into [`Prototype`]s which are kept here once complete
    prototypes: Vec<Prototype>,
}

impl Default for Compiler {
    fn default() -> Self {
        let mut compiler = Self {
            constants: Default::default(),
            compiling: Default::default(),
            prototypes: Default::default(),
        };

        compiler.prime();

        compiler
    }
}

impl Compiler {
    const MAIN_INDEX: Index<Prototype> = Index::new(0);

    /// Prime the compiler by defining a complete 'main' prototype at index zero
    /// that just halts.
    fn prime(&mut self) {
        self.with_prototype(|compiler| {
            compiler
                .active_prototype_mut()
                .set_name(Prototype::MAIN_NAME);

            compiler.emit(Op::Halt, Span::default())
        })
        .unwrap();
    }

    /// Convert the current compiler state into a new [`Object`] that can be
    /// loaded into the runtime.
    pub fn build(&self) -> Result<Object> {
        if !self.compiling.is_empty() {
            return Err(Error::CannotBuildWhileCompiling);
        }

        Ok(Object {
            constants: self.constants.as_vec(),
            prototypes: self.prototypes.clone(),
        })
    }

    /// Push more top-level module code through the compiler.
    pub fn push(&mut self, syntax: &syntax::Module) -> Result<()> {
        if !self.compiling.is_empty() {
            return Err(Error::CanOnlyReopenMain);
        }

        let mut main = self.prototypes[Self::MAIN_INDEX.as_usize()].clone();

        main.reopen()?;
        self.compiling.push(main);

        let old_const_count = self.constants.len();
        let old_proto_count = self.prototypes.len();

        let result = self
            .statement_sequence(syntax)
            .and_then(|()| self.emit(Op::Halt, syntax.span()));

        match result {
            Err(e) => {
                // we need to clean up. We can't recover anything that's
                // partially compiled (yet), and we want to get rid of any
                // now-dead-code prototypes and constants.
                self.compiling.clear();
                self.prototypes.truncate(old_proto_count);
                self.constants.truncate(old_const_count);
                Err(e)
            }

            Ok(()) => {
                debug_assert_eq!(self.compiling.len(), 1, "the only thing compiling after successfully pushing should be the updated main.");

                // we need to overwrite the old main with the new good one.
                let new_main = self.compiling.pop().unwrap();
                self.prototypes[Self::MAIN_INDEX.as_usize()] = new_main;
                Ok(())
            }
        }
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
    pub(crate) fn next_index(&self) -> Index<Op> {
        self.compiling.last().unwrap().code().next_index()
    }

    pub(crate) fn patch(&mut self, index: Index<Op>, op: Op) -> Option<Op> {
        self.active_prototype_mut().code_mut().patch(index, op)
    }

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
        self.begin_prototype()?;
        inner(self)?;
        self.end_prototype()
    }

    fn begin_prototype(&mut self) -> Result<()> {
        if self.compiling.len() + self.prototypes.len() >= u32::MAX as usize {
            return Err(Error::TooManyPrototypes);
        }

        self.compiling.push(Prototype::new());
        Ok(())
    }

    fn end_prototype(&mut self) -> Result<Index<Prototype>> {
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
