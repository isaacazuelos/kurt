//! The virtual machine methods for our runtime.

use compiler::{constant::Constant, index::Index, opcode::Op};

use crate::{error::Result, value::Value, Exit, Runtime};

impl Runtime {
    /// Start the VM up again.
    pub fn run(&mut self) -> Result<Exit> {
        loop {
            let op = self.fetch()?;
            match op {
                Op::Halt => return Ok(Exit::Halt),
                Op::Nop => continue,
                Op::Pop => {
                    self.stack.pop();
                }
                Op::True => self.stack.push(Value::TRUE),
                Op::False => self.stack.push(Value::FALSE),
                Op::Unit => self.stack.push(Value::UNIT),
                Op::LoadConstant(i) => self.load_constant(i)?,
            }
            eprintln!("{:#?}", self);
        }
    }
}

impl Runtime {
    #[inline]
    fn fetch(&mut self) -> Result<Op> {
        let op = self.current_op()?;
        self.pc.increment()?;
        Ok(op)
    }

    #[inline]
    fn load_constant(&mut self, index: Index<Constant>) -> Result<()> {
        let value = self.current_module()?.constant(index)?;
        self.stack.push(value);
        Ok(())
    }
}
