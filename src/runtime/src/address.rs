use std::fmt::{self, Display};

use compiler::{index::Index, opcode::Op, prototype::Prototype};

use crate::{
    error::{Error, Result},
    module::Module,
};

#[derive(Clone, Copy, Debug)]
pub struct Address {
    pub(crate) module: Index<Module>,
    pub(crate) prototype: Index<Prototype>,
    pub(crate) instruction: Index<Op>,
}

impl Address {
    pub(crate) fn new(
        module: Index<Module>,
        prototype: Index<Prototype>,
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
