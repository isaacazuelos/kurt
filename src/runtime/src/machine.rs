//! The virtual machine's big dispatch loop

use compiler::{
    capture::Capture,
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
    memory::{
        closure::Closure,
        list::List,
        upvalue::{Upvalue, UpvalueContents},
    },
    primitives::{Error as PrimitiveError, PrimitiveOperations},
    stack::Stack,
    value::Value,
    Error, Exit, Runtime,
};

impl Runtime {
    /// Start the VM up again.
    pub(crate) fn run(&mut self) -> Result<Exit> {
        loop {
            #[cfg(feature = "trace")]
            self.trace();

            match self.fetch()? {
                // control
                Op::Halt => return Ok(Exit::Halt),
                Op::Nop => continue,

                // stack
                Op::Pop => self.stack.pop(),

                // values
                Op::True => self.stack.push(Value::TRUE),
                Op::False => self.stack.push(Value::FALSE),
                Op::Unit => self.stack.push(Value::UNIT),
                Op::LoadConstant(i) => self.load_constant(i)?,
                Op::LoadLocal(i) => self.load_local(i)?,
                Op::LoadCapture(i) => self.load_capture(i)?,
                Op::LoadClosure(i) => self.load_closure(i)?,
                Op::DefineLocal => self.define_local()?,
                Op::Index => self.binop(Value::index)?,

                // functions
                Op::Call(arg_count) => self.call(arg_count)?,
                Op::Return => self.r#return()?,

                // branching
                Op::Jump(i) => self.jump(i)?,
                Op::BranchFalse(i) => self.branch_false(i)?,

                // logic
                Op::Not => self.unary(Value::not)?,

                // math
                Op::Neg => self.unary(Value::neg)?,
                Op::Add => self.binop(Value::add)?,
                Op::Sub => self.binop(Value::sub)?,
                Op::Mul => self.binop(Value::mul)?,
                Op::Div => self.binop(Value::div)?,
                Op::Pow => self.binop(Value::pow)?,
                Op::Rem => self.binop(Value::rem)?,

                // bitwise
                Op::BitAnd => self.binop(Value::bitand)?,
                Op::BitOr => self.binop(Value::bitor)?,
                Op::BitXOR => self.binop(Value::bitxor)?,
                Op::SHL => self.binop(Value::shl)?,
                Op::SHR => self.binop(Value::shr)?,

                // comparison
                Op::Eq => self.binop(cmp(PrimitiveOperations::eq))?,
                Op::Ne => self.binop(cmp(PrimitiveOperations::ne))?,
                Op::Gt => self.binop(cmp(PrimitiveOperations::gt))?,
                Op::Ge => self.binop(cmp(PrimitiveOperations::ge))?,
                Op::Lt => self.binop(cmp(PrimitiveOperations::lt))?,
                Op::Le => self.binop(cmp(PrimitiveOperations::le))?,

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

    /// The [`LoadCapture`][Op::LoadCapture] instruction pulls a value out of
    /// the currently-executing closure's captures and places it on the top of
    /// the stack.
    #[inline]
    fn load_capture(&mut self, index: Index<Capture>) -> Result<()> {
        let upvalue: UpvalueContents = self
            .stack
            .get_closure(self.bp())
            .ok_or_else(|| Error::StackIndexBelowZero)?
            .use_as::<Closure, _, UpvalueContents>(|c| {
                c.get_capture(index.as_usize())
                    .use_as::<Upvalue, _, _>(|u| Ok(u.contents()))
            })?;

        let value = match upvalue {
            UpvalueContents::Stack(stack_index) => self
                .stack
                .get(stack_index)
                .ok_or_else(|| Error::StackIndexBelowZero)?,
            UpvalueContents::Inline(v) => v,
        };

        self.stack.push(value);
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
        let closure = Value::object(gc_obj);
        self.stack.push(closure);

        let bp = self.bp();

        self.stack
            .get_closure(bp)
            .unwrap()
            .use_as::<Closure, _, ()>(|current_closure| {
                closure.use_as::<Closure, _, ()>(|new_closure| {
                    let cap_len = {
                        let prototype =
                            self.get(module).unwrap().get(index).unwrap();
                        prototype.capture_count()
                    };

                    // now we set up the upvalues
                    for i in 0..cap_len {
                        let (is_local, index) = {
                            let cap = self
                                .get(module)
                                .unwrap()
                                .get(index)
                                .unwrap()
                                .captures()[i];
                            (cap.is_local(), cap.index())
                        };

                        let upvalue = if is_local {
                            let local: Index<Stack> =
                                Index::new(index.as_u32() + self.bp().as_u32());

                            Value::object(self.make_from::<Upvalue, _>(local))
                        } else {
                            current_closure.get_capture(index.as_usize())
                        };

                        new_closure.push_capture(upvalue);
                    }

                    Ok(())
                })
            })?;

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
            .use_as(|c: &Closure| Ok(c.prototype()))?;

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

    /// The [`Return`][Op::Return] instruction returns from a function call,
    /// which means it saves the top of the stack, pops the frame, drops the
    /// values up to the old base pointer, and then puts the result back on the
    /// stack.
    #[inline]
    fn r#return(&mut self) -> Result<()> {
        let frame = self.call_stack.pop().ok_or(Error::CannotReturnFromTop)?;
        let result = self.stack.last();
        self.stack.truncate_to(frame.bp);
        // TODO: are we worried about it collecting result before this?
        self.stack.push(result);
        Ok(())
    }

    /// The [`List(n)`][Op::List] instruction takes the top `n` values on the
    /// stack and makes them the elements of a new list which is let on the top
    /// of the stack.
    #[inline]
    fn list(&mut self, n: u32) -> Result<()> {
        let start = self.stack_frame().len() - n as usize;

        let slice = &self.stack_frame()[start..];
        let vec = Vec::from(slice);
        let list = self.make_from::<List, _>(vec);

        self.stack.set_from_top(n, Value::object(list))?;
        self.stack.truncate_by(n);
        Ok(())
    }

    /// The [`Jump(i)`][Op::Jump] instruction jumps to `i` in the current
    /// prototype. We don't have inter-function or inter-module jumps.
    #[inline]
    fn jump(&mut self, i: Index<Op>) -> Result<()> {
        self.pc_mut().instruction = i;
        Ok(())
    }

    /// The [`BranchFalse(i)`][Op::BranchFalse] instruction consumes teh top of
    /// the stack, and if it [`is_truthy`][PrimitiveOperations::is_truthy] then
    /// continues on. If it's not, then it jumps to `i`.
    #[inline]
    fn branch_false(&mut self, i: Index<Op>) -> Result<()> {
        let truthy = self.stack.last().is_truthy();
        self.stack.pop();

        if !truthy {
            self.jump(i)
        } else {
            Ok(())
        }
    }

    /// Performs a unary operation `op` which applies some function to the value
    /// on the top of the stack, replacing it.
    #[inline]
    fn unary<F, E>(&mut self, op: F) -> Result<()>
    where
        F: Fn(&Value, &mut Runtime) -> std::result::Result<Value, E>,
        E: Into<Error>,
    {
        let arg = self.stack.last();

        let result = op(&arg, self).map_err(Into::into)?;
        self.stack.set_from_top(0, result)
    }

    /// Performs a binary operation `op` which applies some function to the two
    /// values on the top of the stack, replacing them.
    #[inline]
    fn binop<F, E>(&mut self, op: F) -> Result<()>
    where
        F: Fn(&Value, Value, &mut Runtime) -> std::result::Result<Value, E>,
        E: Into<Error>,
    {
        // The order here is tricky, we don't want to remove the operands from
        // the stack (and the GC root set) until after we have the result of
        // `op`.

        let rhs = self.stack.get_from_top(0)?;
        let lhs = self.stack.get_from_top(1)?;

        let result = op(&lhs, rhs, self).map_err(Into::into)?;

        self.stack.set_from_top(1, result)?;
        self.stack.pop();

        Ok(())
    }
}

/// An adapter that makes our comparator operations work more like other binary
/// operators. Just a helper.
#[inline(always)]
fn cmp(
    op: fn(&Value, &Value) -> Option<bool>,
) -> impl Fn(&Value, Value, &mut Runtime) -> std::result::Result<Value, PrimitiveError>
{
    #[inline(always)]
    move |lhs, rhs, _| {
        op(lhs, &rhs).map(Value::bool).ok_or_else(|| {
            PrimitiveError::OperationNotSupported {
                type_name: lhs.type_name(),
                op_name: "cmp",
            }
        })
    }
}
