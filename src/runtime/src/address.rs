use std::fmt::{self, Display};

use compiler::{Function, Index, Module, Op};

use crate::error::{Error, Result};

#[derive(Clone, Copy, Debug)]
pub struct Address {
    pub(crate) module: Index<Module>,
    pub(crate) prototype: Index<Function>,
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
            prototype,
            instruction,
        }
    }

    pub(crate) fn increment(&mut self) -> Result<()> {
        self.instruction =
            self.instruction.next().ok_or(Error::OpIndexOutOfRange)?;
        Ok(())
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "m{:?}/p{:?}/i{:?}",
            self.module, self.prototype, self.instruction
        )
    }
}
