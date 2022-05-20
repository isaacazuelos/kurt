//! A prototype describes a block of runnable code and it's attributes.

use diagnostic::Span;

use crate::{
    error::Result,
    index::{Get, Index},
    internal::{capture::Capture, code::Code, local::Local},
    opcode::Op,
    Constant, Function,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FunctionBuilder {
    name: Option<Index<Constant>>,
    span: Span,
    parameter_count: u32,
    captures: Vec<Capture>,
    code: Code,
    locals: Vec<Local>,
    scopes: Vec<usize>,
}

impl FunctionBuilder {
    /// Crate a prototype for a new closure.
    ///
    /// If you're trying to create one for the top level code, use
    /// [`Prototype::new_main`] instead.
    pub(crate) fn new(span: Span) -> FunctionBuilder {
        FunctionBuilder {
            name: None,
            span,
            parameter_count: 0,
            captures: Vec::new(),
            code: Code::default(),
            locals: Vec::default(),
            scopes: vec![0],
        }
    }

    /// Throw out all the extra context we kept around for compiling and
    /// compress this into a finalized [`Function`].
    pub fn build(&self) -> Function {
        Function {
            name: self.name,
            span: self.span,
            parameter_count: self.parameter_count,
            captures: self.captures.clone(),
            code: self.code.ops().to_owned(),
        }
    }

    /// Set the prototype's name.
    pub(crate) fn set_name(&mut self, name: Index<Constant>) {
        self.name = Some(name);
    }

    /// Set the number of parameters this prototype needs when being called.
    pub(crate) fn set_parameter_count(&mut self, count: u32) {
        self.parameter_count = count;
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

    pub(crate) fn end_scope(&mut self, span: Span) -> usize {
        let total_in_scope = self.locals.len();
        let going_out_of_scope = self.scopes.pop().unwrap();

        debug_assert!(
            !self.scopes.is_empty(),
            "top level function scope should not end."
        );

        debug_assert!(total_in_scope >= going_out_of_scope);

        // self.locals.truncate(total_in_scope - going_out_of_scope);
        for _ in 0..going_out_of_scope {
            let local = self.locals.pop().unwrap();

            if local.is_captured() {
                self.emit(Op::CloseCapture, span).unwrap();
            } else {
                self.emit(Op::Pop, span).unwrap();
            }
        }

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
        enclosing: &mut [FunctionBuilder],
    ) -> Option<Index<Capture>> {
        // The top-level has no captures. At least not yet.
        if enclosing.is_empty() {
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
            // They're different kinds of indexes, but that's okay because a
            // capture index is a local index relative to it's original call
            // frame, which is the one that needs to promote it.
            let index = Index::new(capture.into());
            return Some(self.add_capture(index, false));
        }

        None
    }

    pub(crate) fn mark_as_captured(&mut self, local: Index<Local>) {
        self.locals[local.as_usize()].capture()
    }
}

impl Get<Op> for FunctionBuilder {
    fn get(&self, index: Index<Op>) -> Option<&Op> {
        self.code.get(index)
    }
}
