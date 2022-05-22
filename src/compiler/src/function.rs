use diagnostic::Span;

use common::Index;

use crate::{Capture, FunctionDebug, Local, Op};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Function {
    pub(crate) span: Span,
    pub(crate) parameter_count: u32,
    pub(crate) captures: Vec<Capture>,
    pub(crate) code: Vec<Op>,

    pub(crate) debug_info: Option<FunctionDebug>,
}

impl Function {
    /// The name used for the prototype containing 'main', the top-level code
    /// for a module.
    pub const MAIN_NAME: &'static str = "main";

    /// The name used for functions that don't have names.
    pub const DEFAULT_NAMELESS_NAME: &'static str = "<nameless function>";

    pub const MAX_OPS: usize = Index::<Op>::MAX;

    /// The maximum number of parameters allowed in a function definition
    pub const MAX_PARAMETERS: usize = Index::<Local>::MAX;

    /// The maximum number of arguments allowed in a function call.
    ///
    /// This is limited by the number of parameters a function can access.
    ///
    /// Since parameters are treated the same way as local variable, this ends
    /// up being the same as the max number of local variables.
    pub const MAX_ARGUMENTS: usize = u32::MAX as usize;

    /// The number of parameters required when this function is called.
    pub fn parameter_count(&self) -> u32 {
        self.parameter_count
    }

    /// The number of variable this closure captures.
    pub fn capture_count(&self) -> usize {
        self.captures.len()
    }

    /// A slice containing information about the relative stack positions of all
    /// the values this function captures.
    pub fn captures(&self) -> &[Capture] {
        &self.captures
    }

    pub fn get_capture(&self, index: usize) -> Option<Capture> {
        self.captures.get(index).cloned()
    }

    /// Get the instruction at the given instruction index. Returns `None` if
    /// the instruction is out of range.
    pub fn get(&self, index: Index<Op>) -> Option<Op> {
        self.code.get(index.as_usize()).cloned()
    }

    /// The span in the source code where this function was defined.
    pub fn span(&self) -> Span {
        self.span
    }

    /// The debug info for this module.
    pub fn debug_info(&self) -> Option<&FunctionDebug> {
        self.debug_info.as_ref()
    }

    /// Throw away the extra debug info this function carries.
    pub fn strip_debug(&mut self) {
        self.debug_info = None;
    }

    /// Kind of a gross hack.
    ///
    /// Main needs to end with a Halt, so this pushes that halt, and updates teh
    /// debug info's spans to match the last instructions span.
    pub(crate) fn close_with_halt_for_main(&mut self) {
        self.code.push(Op::Halt);

        if let Some(debug_info) = &mut self.debug_info {
            let last_span =
                debug_info.code_spans.last().cloned().unwrap_or_default();
            debug_info.code_spans.push(last_span);
        }
    }
}
