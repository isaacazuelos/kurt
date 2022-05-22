use compiler::{Function, Index, Module, Op};

use crate::error::{Error, Result};

#[derive(Clone, Copy, Debug)]
pub struct Address {
    pub(crate) module: Index<Module>,
    pub(crate) function: Index<Function>,
    pub(crate) instruction: Index<Op>,
}

impl Address {
    pub(crate) fn new(
        module: Index<Module>,
        prototype: Index<Function>,
        instruction: Index<Op>,
    ) -> Address {
        Address {
            module,
            function: prototype,
            instruction,
        }
    }

    pub(crate) fn increment(&mut self) -> Result<()> {
        self.instruction =
            self.instruction.next().ok_or(Error::OpIndexOutOfRange)?;
        Ok(())
    }
}
