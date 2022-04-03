//! A prototype describes a block of runnable code and it's attributes.

use diagnostic::Span;

use crate::{
    code::Code,
    error::Result,
    index::{Index, Indexable},
    opcode::Op,
};

#[derive(Debug, Clone)]
pub struct Prototype {
    name: Option<String>,
    code: Code,
}

impl Prototype {
    const MAIN_NAME: &'static str = "main";

    /// Crate a prototype for a new closure.
    ///
    /// If you're trying to create one for the top level code of a module, use
    /// [`Prototype::new_main`] instead.
    pub(crate) fn new() -> Prototype {
        Prototype {
            name: None,
            code: Code::default(),
        }
    }

    /// Create a new prototype for the top level of a module.
    pub fn new_main() -> Prototype {
        let mut proto = Prototype::new();
        proto.name = Some(String::from(Prototype::MAIN_NAME));
        proto
    }

    /// Emit into this prototype's code segment.
    pub(crate) fn emit(&mut self, op: Op, span: Span) -> Result<()> {
        self.code.emit(op, span)
    }

    /// Emit, but for synthetic opcodes, i.e. ones which don't have a meaningful
    /// location in the original source code.
    pub(crate) fn emit_synthetic(&mut self, op: Op) -> Result<()> {
        self.code.emit_synthetic(op)
    }

    /// Is this prototype empty?
    ///
    /// A prototype is empty when no code has been compiled for it yet.
    pub(crate) fn is_empty(&self) -> bool {
        self.code.is_empty()
    }

    pub fn span_for_op(&self, index: Index<Op>) -> Option<Span> {
        self.code.get_span(index)
    }
}

impl Indexable<Op> for Prototype {
    fn get(&self, index: Index<Op>) -> Option<&Op> {
        self.code.get(index)
    }
}
