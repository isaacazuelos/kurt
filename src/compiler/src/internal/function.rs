//! A function builder describes a block of runnable code and it's attributes as
//! it's being compiled.

use diagnostic::Span;

use common::{Get, Index};

use crate::{
    error::{Error, Result},
    internal::{capture::Capture, code::Code, local::Local},
    opcode::Op,
    Function, FunctionDebug,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FunctionBuilder {
    name: Option<String>,
    span: Span,
    parameter_count: u32,
    captures: Vec<Capture>,
    code: Code,
    locals: Vec<Local>,
    scopes: Vec<usize>,
}

impl FunctionBuilder {
    /// Crate a builder for a new function.
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
        let debug_info = FunctionDebug::new(self);

        Function {
            span: self.span,
            parameter_count: self.parameter_count,
            captures: self.captures.clone(),
            code: self.code.ops().to_owned(),

            debug_info,
        }
    }

    /// Get teh function's name, if known.
    pub(crate) fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the functions's name.
    pub(crate) fn set_name(&mut self, name: Option<&str>) {
        self.name = name.map(ToOwned::to_owned)
    }

    /// The number of parameters this function
    pub(crate) fn parameter_count(&self) -> u32 {
        self.parameter_count
    }

    pub(crate) fn parameters(&self) -> &[Local] {
        &self.locals[0..(self.parameter_count() as usize)]
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
        span: Span,
    ) -> Result<Index<Capture>> {
        // reuse if already captured
        for (i, capture) in self.captures.iter().enumerate() {
            if capture.index() == local_index && capture.is_local() == is_local
            {
                // We only add things to self.captures below, and check it there.
                return Ok(Index::new(i as u32));
            }
        }

        let capture_index = self.captures.len();
        if capture_index >= Function::MAX_BINDINGS {
            Err(Error::TooManyLocals(span))
        } else {
            self.captures.push(Capture::new(local_index, is_local));
            Ok(Index::new(capture_index as u32))
        }
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

    // Err means an error occurred, whereas Ok(None) means no error but there's
    // no capture found.
    pub(crate) fn resolve_capture(
        &mut self,
        name: &str,
        span: Span,
        enclosing: &mut [FunctionBuilder],
    ) -> Result<Option<Index<Capture>>> {
        // we just checked if it's empty, so unwrap is safe.
        if let Some((next, enclosing_next)) = enclosing.split_last_mut() {
            // If it's a local, turn it into a capture
            if let Some(local) = next.resolve_local(name) {
                next.mark_as_captured(local);
                let index = self.add_capture(local, true, span)?;
                return Ok(Some(index));
            }

            // If it's a capture of some enclosing scope, capture that.
            if let Some(capture) =
                next.resolve_capture(name, span, enclosing_next)?
            {
                // They're different kinds of indexes, but that's okay because a
                // capture index is a local index relative to it's original call
                // frame, which is the one that needs to promote it.
                let local_index = Index::new(capture.into());
                let capture_index =
                    self.add_capture(local_index, false, span)?;
                return Ok(Some(capture_index));
            }
        }

        Ok(None)
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
