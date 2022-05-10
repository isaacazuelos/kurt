//! A prototype describes a block of runnable code and it's attributes.

use diagnostic::Span;

use crate::{
    capture::Capture,
    code::Code,
    error::Result,
    index::{Get, Index},
    local::Local,
    opcode::Op,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Prototype {
    name: Option<String>,
    span: Span,
    parameter_count: u32,
    captures: Vec<Capture>,
    code: Code,
    locals: Vec<Local>,
    scopes: Vec<usize>,
}

impl Prototype {
    /// The name used for the prototype containing 'main', the top-level code
    /// for a module.
    pub const MAIN_NAME: &'static str = "main";

    /// The maximum number of arguments allowed in a function call.
    pub const MAX_ARGUMENTS: usize = u32::MAX as usize;

    /// The maximum number of parameters allowed in a function.
    pub const MAX_PARAMETERS: usize = u32::MAX as usize;

    /// Crate a prototype for a new closure.
    ///
    /// If you're trying to create one for the top level code, use
    /// [`Prototype::new_main`] instead.
    pub(crate) fn new(span: Span) -> Prototype {
        Prototype {
            name: None,
            span,
            parameter_count: 0,
            captures: Vec::new(),
            code: Code::default(),
            locals: Vec::default(),
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
    pub fn parameter_count(&self) -> u32 {
        self.parameter_count
    }

    /// Get the prototype's maximum capture count.
    pub fn capture_count(&self) -> usize {
        self.captures.len()
    }

    /// The captures this prototype has.
    pub fn captures(&self) -> &[Capture] {
        &self.captures
    }

    /// Set the number of parameters this prototype needs when being called.
    pub(crate) fn set_parameter_count(&mut self, count: u32) {
        self.parameter_count = count;
    }

    /// A view of the local bindings which represent the parameters.
    pub(crate) fn parameters(&self) -> &[Local] {
        &self.locals[0..self.parameter_count as usize]
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

    /// Get a mutable reference to the code listing for this prototype.
    pub(crate) fn code_mut(&mut self) -> &mut Code {
        &mut self.code
    }

    pub(crate) fn begin_scope(&mut self) {
        self.scopes.push(0);
    }

    pub(crate) fn end_scope(&mut self) -> usize {
        let total_in_scope = self.locals.len();
        let going_out_of_scope = self.scopes.pop().unwrap();

        debug_assert!(
            !self.scopes.is_empty(),
            "top level function scope should not end."
        );

        self.locals.truncate(total_in_scope - going_out_of_scope);
        going_out_of_scope
    }

    /// Bind a [`Local`] in the current scope.
    pub(crate) fn bind_local(&mut self, local: Local) {
        // TODO: Error::TooManyLocals

        if let Some(count) = self.scopes.last_mut() {
            *count += 1;
        }

        self.locals.push(local);
    }

    pub(crate) fn add_capture(
        &mut self,
        local_index: Index<Local>,
        is_local: bool,
    ) -> Index<Capture> {
        // reuse if already captured
        for (i, capture) in self.captures.iter().enumerate() {
            if capture.index() == local_index && capture.is_local() == is_local
            {
                return Index::new(i as u32);
            }
        }

        // TODO: Error::TooManyLocals

        let capture_index = self.captures.len() as u32;
        self.captures.push(Capture::new(local_index, is_local));

        Index::new(capture_index)
    }

    /// Return the [`Index<Local>`] for a local variable with the given name, if
    /// one is in scope.
    pub(crate) fn resolve_local(&mut self, name: &str) -> Option<Index<Local>> {
        println!("looking for a local named {} in {:?}", name, &self.locals);

        // the rev is so we find more recently bound locals faster than less
        // recently bound ones, and ensures that shadowing works by finding the
        // most-recent binding with the given name.
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.as_str() == name {
                return Some(Index::new(i as _));
            }
        }

        None
    }

    pub(crate) fn resolve_capture(
        &mut self,
        name: &str,
        enclosing: &mut [Prototype],
    ) -> Option<Index<Capture>> {
        dbg!(&self, name, enclosing.len());

        // The top-level has no captures. At least not yet.
        if enclosing == [] {
            return None;
        }

        let (next, enclosing_next) = enclosing.split_last_mut()?;

        // If it's a local, turn it into a capture
        if let Some(local) = next.resolve_local(name) {
            next.mark_as_captured(local);
            return Some(self.add_capture(local, true));
        }

        // If it's a capture of some enclosing scope, capture that.
        if let Some(capture) = next.resolve_capture(name, enclosing_next) {
            println!("found capture, adding");
            // They're different kinds of indexes, but that's okay because a
            // capture index is a local index relative to it's original call
            // frame, which is the one that needs to promote it.
            let index = Index::new(capture.as_u32());
            return Some(self.add_capture(index, false));
        }

        println!("not found");

        None
    }

    /// Reopen the prototype.
    ///
    /// This is done to allow the compiler to push more code through. This can
    /// only be done if the prototype ends in `Halt`.
    pub(crate) fn reopen(&mut self) {
        match self.code().last() {
            Some(Op::Halt) => {
                let index = self.code().next_index().pred_saturating();
                self.code_mut().patch(index, Op::Nop).expect(
                    "compiler could not patch Op::Halt with Op::Nop to reopen",
                );
            }
            None => {}
            Some(op) => panic!(
                "compiler can only reopen module at Op::Halt but found {}",
                op
            ),
        }
    }

    pub(crate) fn mark_as_captured(&mut self, local: Index<Local>) {
        self.locals[local.as_usize()].capture()
    }
}

impl Get<Op> for Prototype {
    fn get(&self, index: Index<Op>) -> Option<&Op> {
        self.code.get(index)
    }
}
