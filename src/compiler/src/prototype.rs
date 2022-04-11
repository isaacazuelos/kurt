//! A prototype describes a block of runnable code and it's attributes.

use diagnostic::Span;

use crate::{
    code::Code,
    error::Result,
    index::{Index, Indexable},
    local::Local,
    opcode::Op,
};

#[derive(Debug, Clone)]
pub struct Prototype {
    name: Option<String>,
    parameter_count: usize,
    code: Code,
    bindings: Vec<Local>,
    scopes: Vec<usize>,
}

impl Prototype {
    /// The name used for the prototype containing 'main', the top-level code
    /// for a module.
    pub const MAIN_NAME: &'static str = "main";

    /// Crate a prototype for a new closure.
    ///
    /// If you're trying to create one for the top level code, use
    /// [`Prototype::new_main`] instead.
    pub(crate) fn new() -> Prototype {
        Prototype {
            name: None,
            parameter_count: 0,
            code: Code::default(),
            bindings: Vec::default(),
            scopes: vec![0],
        }
    }

    /// The name of the module, if it has one.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the prototype's name.
    pub(crate) fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Get the prototype's parameter count.
    pub fn parameter_count(&self) -> usize {
        self.parameter_count
    }

    /// Set the number of parameters this prototype needs when being called.
    pub(crate) fn set_parameter_count(&mut self, count: usize) {
        self.parameter_count = count;
    }

    /// The span which created a specific opcode.
    pub fn span_for_op(&self, index: Index<Op>) -> Option<Span> {
        self.code.get_span(index)
    }

    /// Is this prototype empty, as in no code has been compiled to it?
    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }

    /// Emit into this prototype's code segment.
    pub(crate) fn emit(&mut self, op: Op, span: Span) -> Result<()> {
        self.code.emit(op, span)
    }

    /// The code listing for this prototype.
    pub(crate) fn code(&self) -> &Code {
        &self.code
    }

    pub(crate) fn begin_scope(&mut self) {
        self.scopes.push(0);
    }

    pub(crate) fn end_scope(&mut self) -> usize {
        let total_in_scope = self.bindings.len();
        let going_out_of_scope = self.scopes.pop().unwrap();

        debug_assert!(
            !self.scopes.is_empty(),
            "top level function scope should not end."
        );

        self.bindings.truncate(total_in_scope - going_out_of_scope);
        going_out_of_scope
    }

    /// Bind a [`Local`] in the current scope.
    pub(crate) fn bind_local(&mut self, local: Local) {
        if let Some(count) = self.scopes.last_mut() {
            *count += 1;
        }

        self.bindings.push(local);
    }

    pub(crate) fn resolve_local(&mut self, name: &str) -> Option<Index<Local>> {
        // the rev is so we find more recently bound locals faster than less
        // recently bound ones, and ensures that shadowing works by finding the
        // most-recent binding with the given name.
        for (i, local) in self.bindings.iter().enumerate().rev() {
            if local.as_str() == name {
                return Some(Index::new(i as _));
            }
        }

        None
    }
}

impl Indexable<Op> for Prototype {
    fn get(&self, index: Index<Op>) -> Option<&Op> {
        self.code.get(index)
    }
}
