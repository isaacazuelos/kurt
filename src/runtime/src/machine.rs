//! The virtual machine methods for our runtime.

use compiler::{
    constant::Constant,
    index::{Get, Index},
    local::Local,
    opcode::Op,
    prototype::Prototype,
};

use crate::{
    address::Address,
    call_stack::CallFrame,
    error::Result,
    memory::{closure::Closure, list::List},
    primitives::PrimitiveOperations,
    value::Value,
    Error, Exit, Runtime,
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
                // control
                Op::Halt => return Ok(Exit::Halt),
                Op::Yield => return Ok(Exit::Yield),
                Op::Nop => continue,
                // stack
                Op::Pop => {
                    self.stack.pop();
                }
                // values
                Op::True => self.stack.push(Value::TRUE),
                Op::False => self.stack.push(Value::FALSE),
                Op::Unit => self.stack.push(Value::UNIT),
                Op::LoadConstant(i) => self.load_constant(i)?,
                Op::LoadLocal(i) => self.load_local(i)?,
                Op::LoadClosure(i) => self.load_closure(i)?,
                Op::DefineLocal => self.define_local()?,
                Op::Subscript => self.subscript()?,
                // functions
                Op::Call(arg_count) => self.call(arg_count)?,
                Op::Return => self.return_op()?,
                // branching
                Op::Jump(i) => self.jump(i)?,
                Op::BranchFalse(i) => self.branch_false(i)?,
                // logic
                Op::Not => self.not()?,
                // math
                Op::Neg => self.neg()?,
                Op::Add => self.binop(Value::add)?,
                Op::Sub => self.binop(Value::sub)?,
                Op::Mul => self.binop(Value::mul)?,
                Op::Div => self.binop(Value::div)?,
                Op::Pow => self.binop(Value::pow)?,
                Op::Rem => self.binop(Value::rem)?,
                // // bitwise
                Op::BitAnd => self.binop(Value::bitand)?,
                Op::BitOr => self.binop(Value::bitor)?,
                Op::BitXOR => self.binop(Value::bitxor)?,
                Op::SHL => self.binop(Value::shl)?,
                Op::SHR => self.binop(Value::shr)?,
                // comparison
                Op::Eq => self.eq()?,
                Op::Ne => {
                    self.eq()?;
                    self.not()?
                }
                Op::Gt => self.cmp(<Value as PrimitiveOperations>::gt)?,
                Op::Ge => self.cmp(<Value as PrimitiveOperations>::ge)?,
                Op::Lt => self.cmp(<Value as PrimitiveOperations>::lt)?,
                Op::Le => self.cmp(<Value as PrimitiveOperations>::le)?,
                // temporary
                Op::List(n) => self.list(n)?,
            }
        }
    }

    #[inline]
    fn fetch(&mut self) -> Result<Op> {
        let op = self.op()?;
        self.pc_mut().increment()?;
        Ok(op)
    }
}

impl Runtime {
    /// The [`LoadConstant`][Op::LoadConstant] instruction loads a constant from
    /// the current module's constant pool using the given `index` and places it
    /// on the stop of the stack.
    #[inline]
    fn load_constant(&mut self, index: Index<Constant>) -> Result<()> {
        let value = *self
            .get(self.pc().module)
            .ok_or(Error::ModuleIndexOutOfRange)?
            .get(index)
            .ok_or(Error::ConstantIndexOutOfRange)?;

        self.stack.push(value);
        Ok(())
    }

    /// The [`LoadLocal`][Op::LoadLocal] instruction loads a copies the [`Value`]
    /// of a local variable places it on the stop of the stack.
    ///
    /// Local variables are indexed up from the base pointer.
    #[inline]
    fn load_local(&mut self, index: Index<Local>) -> Result<()> {
        let local = self.stack.get_local(self.bp(), index)?;
        self.stack.push(local);
        Ok(())
    }

    /// The [`DefineLocal`][Op::DefineLocal] instruction increments the top of the
    /// stack by pushing a `()`. This has the effect of leaving the value
    /// previous on the top of the stack around.
    #[inline]
    fn define_local(&mut self) -> Result<()> {
        self.stack.push(Value::UNIT);
        Ok(())
    }

    /// The [`LoadClosure`][Op::LoadClosure] instruction creates an instance of
    /// the closure described by the indexed [`Prototype`] in the current
    /// module, and leaves it on the stack.
    #[inline]
    fn load_closure(&mut self, index: Index<Prototype>) -> Result<()> {
        let module = self.pc().module;
        let gc_obj = self.make_from::<Closure, _>((module, index));
        self.stack.push(Value::object(gc_obj));
        Ok(())
    }

    /// The [`Subscript`][Op::Subscript] instruction
    fn subscript(&mut self) -> Result<()> {
        let index = self.stack.pop();
        let target = self.stack.pop();
        let result = target.index(index, self).map_err(Into::into)?;
        self.stack.push(result);
        Ok(())
    }

    /// The [`Call`][Op::Call] instruction calls a function passing the
    /// indicated number of arguments. This is done by creating and pushing a
    /// new frame on the [`CallStack`][crate::call_stack::CallStack].
    ///
    /// The target of the function call is the value that's just 'below' the
    /// arguments on the stack.
    #[inline]
    fn call(&mut self, arg_count: u32) -> Result<()> {
        let module = self.pc().module;

        let prototype = self
            .stack
            .get_from_top(arg_count)?
            .use_as(|c: &Closure| Ok(c.prototype()))
            .map_err(Into::into)?;

        match self.get(prototype) {
            Some(p) if p.parameter_count() == arg_count => Ok(()),
            Some(_) => Err(Error::InvalidArgCount),
            None => Err(Error::PrototypeIndexOutOfRange),
        }?;

        let pc = Address::new(module, prototype, Index::START);
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

    #[inline]
    fn list(&mut self, n: u32) -> Result<()> {
        let mut vec = Vec::with_capacity(n as _);

        for _ in 0..n {
            vec.push(self.stack.pop())
        }

        vec.reverse();

        let list = self.make_from::<List, _>(vec);

        self.stack.push(Value::object(list));
        Ok(())
    }

    #[inline]
    fn jump(&mut self, i: Index<Op>) -> Result<()> {
        self.pc_mut().instruction = i;
        Ok(())
    }

    #[inline]
    fn branch_false(&mut self, i: Index<Op>) -> Result<()> {
        if self.stack.pop() == Value::FALSE {
            self.jump(i)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn not(&mut self) -> Result<()> {
        let value = self.stack.pop();
        let result = value.not(self).map_err(Into::into)?;
        self.stack.push(result);
        Ok(())
    }

    #[inline]
    fn neg(&mut self) -> Result<()> {
        let value = self.stack.pop();
        let result = value.neg(self).map_err(Into::into)?;
        self.stack.push(result);
        Ok(())
    }

    #[inline]
    fn binop<F, E>(&mut self, op: F) -> Result<()>
    where
        F: Fn(&Value, Value, &mut Runtime) -> std::result::Result<Value, E>,
        E: Into<Error>,
    {
        // TODO: We should not remove these form teh stack until after calling
        //       `op` in case it triggers garbage collection.
        let rhs = self.stack.pop();
        let lhs = self.stack.pop();

        let result = op(&lhs, rhs, self).map_err(Into::into)?;
        self.stack.push(result);
        Ok(())
    }

    #[inline]
    fn eq(&mut self) -> Result<()> where {
        // TODO: We should not remove these form teh stack until after calling
        //       `op` in case it triggers garbage collection.
        let rhs = self.stack.pop();
        let lhs = self.stack.pop();

        self.stack.push(Value::bool(lhs == rhs));
        Ok(())
    }

    #[inline]
    fn cmp<F>(&mut self, op: F) -> Result<()>
    where
        F: Fn(&Value, &Value) -> Option<bool>,
    {
        // TODO: We should not remove these form teh stack until after calling
        //       `op` in case it triggers garbage collection.
        let rhs = self.stack.pop();
        let lhs = self.stack.pop();

        // TODO: Things which aren't comparable return `false` when compared to
        //       anything. Not sure this is ideal.
        let result = op(&lhs, &rhs).unwrap_or(false);

        self.stack.push(Value::bool(result));
        Ok(())
    }
}
