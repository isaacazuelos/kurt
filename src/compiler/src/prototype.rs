//! A prototype describes a block of runnable code and it's attributes.

use diagnostic::Span;

use crate::{code::Code, error::Result, opcode::Op};

#[derive(Debug, Clone)]
pub struct Prototype {
    code: Code,
}

impl Prototype {
    /// Crate a prototype for a new closure.
    ///
    /// If you're trying to create one for the top level of a module, use
    /// [`Prototype::new_top_level`] instead.
    pub(crate) fn new() -> Prototype {
        Prototype {
            code: Code::default(),
        }
    }

    /// Create a new prototype for the top level of a module.
    pub(crate) fn new_top_level() -> Prototype {
        // no differences, yet.
        Prototype::new()
    }

    /// Emit into this prototypes code segment.
    pub(crate) fn emit(&mut self, op: Op, span: Span) -> Result<()> {
        self.code.emit(op, span);
        Ok(())
    }
}
