//! The virtual machine methods for our runtime.

use compiler::{
    constant::Constant, index::Index, local::Local, opcode::Op,
    prototype::Prototype,
};

use crate::{
    address::Address, call_stack::CallFrame, error::Result,
    memory::closure::Closure, value::Value, Error, Exit, Runtime,
};

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
                Op::Return => self.return_op()?,
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
    fn load_closure(&mut self, index: Index<Prototype>) -> Result<()> {
        let module = self.current_frame().pc.module;
        let gc_obj = self.make_from::<Closure, _>((module, index));
        self.stack.push(Value::object(gc_obj));
        Ok(())
    }

    #[inline]
    fn call(&mut self, arg_count: u32) -> Result<()> {
        // TODO: checks that we're calling a closure, and that the param count works.

        let target_module = self.current_frame().pc.module;
        let target_prototype = self
            .stack
            .get_from_top(arg_count)?
            .as_object()
            .and_then(|o| {
                o.deref().downcast::<Closure>().map(|c| c.prototype_index())
            })
            .ok_or(Error::CanOnlyCallClosures)?;

        let pc = Address::new(target_module, target_prototype, Index::START);
        let bp = self.stack.index_from_top(arg_count);

        let new_frame = CallFrame::new(pc, bp);
        self.call_stack.push(new_frame);

        Ok(())
    }

    #[inline]
    fn return_op(&mut self) -> Result<()> {
        let frame = self.call_stack.pop().ok_or(Error::CannotReturnFromTop)?;
        let result = self.stack.pop();
        self.stack.truncate_to(frame.bp);
        self.stack.push(result);

        Ok(())
    }
}
