use common::Index;
use compiler::Op;
use diagnostic::Span;

use crate::{
    classes::Closure,
    error::{Error, Result},
    memory::Gc,
};

#[derive(Clone, Debug)]
pub struct Address {
    pub(crate) closure: Gc<Closure>,
    pub(crate) instruction: Index<Op>,
}

impl Address {
    pub(crate) fn new(closure: Gc<Closure>, instruction: Index<Op>) -> Address {
        Address {
            closure,
            instruction,
        }
    }

    pub(crate) fn increment(&mut self) -> Result<()> {
        self.instruction =
            self.instruction.next().ok_or(Error::OpIndexOutOfRange)?;
        Ok(())
    }

    pub(crate) fn span(&self) -> Option<Span> {
        let instruction = self.instruction;

        self.closure
            .prototype()
            .debug_info()
            .and_then(|i| i.span_of(instruction))
    }
}
