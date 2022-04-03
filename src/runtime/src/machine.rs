//! The virtual machine methods for our runtime.

use compiler::{constant::Constant, index::Index, local::Local, opcode::Op};

use crate::{error::Result, value::Value, Exit, Runtime};

impl Runtime {
    /// Start the VM up again.
    pub fn run(&mut self) -> Result<Exit> {
        loop {
            if self.tracing {
                self.trace();
            }

            let op = self.fetch()?;

            match op {
                Op::Halt => return Ok(Exit::Halt),
                Op::Yield => return Ok(Exit::Yield),

                Op::Nop => continue,
                Op::Pop => {
                    self.stack.pop();
                }

                Op::True => self.stack.push(Value::TRUE),
                Op::False => self.stack.push(Value::FALSE),
                Op::Unit => self.stack.push(Value::UNIT),

                Op::LoadConstant(i) => self.load_constant(i)?,

                Op::LoadLocal(i) => self.load_local(i)?,
                Op::DefineLocal => self.define_local()?,
            }
        }
    }
}

impl Runtime {
    #[inline]
    fn fetch(&mut self) -> Result<Op> {
        let op = self.current_op()?;
        self.current_frame_mut().pc.increment()?;
        Ok(op)
    }

    #[inline]
    fn load_constant(&mut self, index: Index<Constant>) -> Result<()> {
        let value = self.current_module()?.constant(index)?;
        self.stack.push(value);
        Ok(())
    }

    #[inline]
    fn load_local(&mut self, local: Index<Local>) -> Result<()> {
        let base = self.current_frame().bp;
        let local = self.stack.get_local(base, local)?;
        self.stack.push(local);
        Ok(())
    }

    #[inline]
    fn define_local(&mut self) -> Result<()> {
        self.stack.push(Value::UNIT);
        Ok(())
    }
}
