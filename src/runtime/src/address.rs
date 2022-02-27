use compiler::index::Index;
use compiler::opcode::Op;
use compiler::prototype::Prototype;

use crate::error::{Error, Result};

#[derive(Clone, Copy, Debug)]
pub struct Address {
    pub(crate) prototype: Index<Prototype>,
    pub(crate) instruction: Index<Op>,
}

impl Default for Address {
    fn default() -> Self {
        Address {
            prototype: Index::MAIN,
            instruction: Index::START,
        }
    }
}

impl Address {
    pub(crate) fn increment(&mut self) -> Result<()> {
        self.instruction =
            self.instruction.next().ok_or(Error::OpIndexOutOfRange)?;
        Ok(())
    }
}
