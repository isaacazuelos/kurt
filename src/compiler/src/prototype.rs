//! A prototype describes a block of runnable code and it's attributes.

use diagnostic::Span;

use crate::{code::Code, error::Result, opcode::Op};

#[derive(Debug, Clone)]
pub struct Prototype {
    name: Option<String>,
    code: Code,
}

impl Prototype {
    const MAIN_NAME: &'static str = "main";

    /// Crate a prototype for a new closure.
    ///
    /// If you're trying to create one for the top level of a module, use
    /// [`Prototype::new_top_level`] instead.
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

    /// Emit into this prototypes code segment.
    pub(crate) fn emit(&mut self, op: Op, span: Span) -> Result<()> {
        self.code.emit(op, span);
        Ok(())
    }
}
