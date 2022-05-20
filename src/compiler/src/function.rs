use diagnostic::Span;

use crate::{Capture, Constant, Index, Local, Op};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Function {
    pub(crate) name: Option<Index<Constant>>,
    pub(crate) span: Span,
    pub(crate) parameter_count: u32,
    pub(crate) captures: Vec<Capture>,
    pub(crate) code: Vec<Op>,
}

impl Function {
    /// The name used for the prototype containing 'main', the top-level code
    /// for a module.
    pub const MAIN_NAME: &'static str = "main";

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

    /// The name of the function (if it was named), as a constant index for the
    /// [`Module`] it was defined inside.
    pub fn name(&self) -> Option<Index<Constant>> {
        self.name
    }

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

    /// Get the instruction at the given instruction index. Returns `None` if
    /// the instruction is out of range.
    pub fn get(&self, index: Index<Op>) -> Option<Op> {
        self.code.get(index.as_usize()).cloned()
    }

    /// The span in the source code where this function was defined.
    pub fn span(&self) -> Span {
        self.span
    }
}
