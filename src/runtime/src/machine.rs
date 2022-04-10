//! The virtual machine methods for our runtime.

use compiler::{
    constant::Constant, index::Index, local::Local, opcode::Op,
    prototype::Prototype,
};

use crate::{error::Result, value::Value, Exit, Runtime};

impl Runtime {
    /// Start the VM up again.
    pub(crate) fn run(&mut self) -> Result<Exit> {
        loop {
            if self.tracing {
                self.trace();
            }

            let op = self.fetch()?;

            match op {
                Op::Halt => return Ok(Exit::Halt),
                Op::Yield => return Ok(Exit::Yield),
                Op::Return => self.return_op()?,

                Op::Nop => continue,
                Op::Pop => {
                    self.stack.pop();
                }

                Op::True => self.stack.push(Value::TRUE),
                Op::False => self.stack.push(Value::FALSE),
                Op::Unit => self.stack.push(Value::UNIT),

                Op::LoadConstant(i) => self.load_constant(i)?,
                Op::LoadLocal(i) => self.load_local(i)?,
                Op::LoadClosure(i) => self.load_closure(i)?,

                Op::DefineLocal => self.define_local()?,

                Op::Call(arg_count) => self.call(arg_count)?,
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
    fn load_local(&mut self, index: Index<Local>) -> Result<()> {
        let base = self.current_frame().bp;
        let local = self.stack.get_local(base, index)?;
        self.stack.push(local);
        Ok(())
    }

    #[inline]
    fn define_local(&mut self) -> Result<()> {
        self.stack.push(Value::UNIT);
        Ok(())
    }

    #[inline]
    fn load_closure(&mut self, _index: Index<Prototype>) -> Result<()> {
        todo!("Closures not yet implemented")
    }

    #[inline]
    fn call(&mut self, _arg_count: u32) -> Result<()> {
        todo!("Calls not yet implemented")
    }

    #[inline]
    fn return_op(&mut self) -> Result<()> {
        todo!("Calls not yet implemented")
    }
}
