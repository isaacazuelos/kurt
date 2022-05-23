use common::Index;
use compiler::{Function, Op};

use crate::{
    classes::Module,
    error::{Error, Result},
    memory::Gc,
};

#[derive(Clone, Copy, Debug)]
pub struct Address {
    pub(crate) module: Gc<Module>,
    pub(crate) function: Index<Function>,
    pub(crate) instruction: Index<Op>,
}

impl Address {
    pub(crate) fn new(
        module: Gc<Module>,
        function: Index<Function>,
        instruction: Index<Op>,
    ) -> Address {
        Address {
            module,
            function,
            instruction,
        }
    }

    pub(crate) fn increment(&mut self) -> Result<()> {
        self.instruction =
            self.instruction.next().ok_or(Error::OpIndexOutOfRange)?;
        Ok(())
    }
}
