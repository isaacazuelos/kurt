//! The virtual machine's big dispatch loop

use common::{i48, Get, Index};
use compiler::{Capture, Constant, Function, Local, Op};

use crate::{
    classes::{Closure, List},
    error::Result,
    memory::Gc,
    primitives::PrimitiveOperations,
    value::Value,
    vm::{CallFrame, Exit, ValueStack},
    Error, VirtualMachine,
};

impl VirtualMachine {
    /// Start the VM up again.
    pub(crate) fn run(&mut self) -> Result<Exit> {
        loop {
            #[cfg(feature = "trace")]
            self.trace();

            match self.fetch()? {
                // control
                Op::Halt => return self.halt(),
                Op::Nop => continue,

                // stack
                Op::Pop => self.value_stack.pop(),
                Op::Close(n) => self.close(n)?,

                // values
                Op::True => self.value_stack.push(Value::TRUE),
                Op::False => self.value_stack.push(Value::FALSE),
                Op::Unit => self.value_stack.push(Value::UNIT),
                Op::U48(n) => self.value_stack.push(Value::int(
                    i48::try_from(n).map_err(|_| Error::NumberTooBig)?,
                )),
                Op::I48(n) => self.value_stack.push(Value::from(n)),

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
        let op = self.op().expect("op code not in range");
        self.pc_mut().saturating_increment();
        Ok(op)
    }

    /// Closes any open upvalues which occur in the open list with a stack index
    /// above or at `top`.
    #[inline]
    fn close_captures_up_to(&mut self, top: Index<ValueStack>) {
        while let Some(cell) = self.open_captures.pop_up_to(top) {
            let value = self
                .value_stack
                .get(
                    cell.stack_index()
                        .expect("must be open, just got from open list"),
                )
                .expect("open capture cell past end of stack");

            cell.close(value);
        }
    }
}

impl VirtualMachine {
    /// The [`Halt`][Op::Halt] instruction stops the VM, and moves the
    /// instruction pointer back one so it's back on the `Halt` opcode.
    #[inline]
    fn halt(&mut self) -> Result<Exit> {
        self.pc_mut().saturating_decrement();
        Ok(Exit::Halt)
    }

    /// The [`Close`][Op::Close] instructions slides the value on the top of the
    /// stack back `n` spaces, and closes any open captures along the way.
    #[inline]
    fn close(&mut self, n: u32) -> Result<()> {
        // We need to add one to this index since we're keeping the current top
        // of the stack and sliding it down.
        let new_top = self.value_stack().index_from_top(n);

        self.close_captures_up_to(new_top);

        let kept = self.value_stack().last();
        self.value_stack.set_from_top(n, kept);
        self.value_stack.truncate_by(n);

        Ok(())
    }

    /// The [`LoadConstant`][Op::LoadConstant] instruction loads a constant from
    /// the current module's constant pool using the given `index` and places it
    /// on the stop of the stack.
    #[inline]
    fn load_constant(&mut self, index: Index<Constant>) -> Result<()> {
        let constant = self
            .current_closure()
            .module()
            .get(index)
            .expect("constant index out of range")
            .clone();

        let value = self.inflate(&constant)?;

        self.value_stack.push(value);
        Ok(())
    }

    /// The [`LoadLocal`][Op::LoadLocal] instruction loads a copies the [`Value`]
    /// of a local variable places it on the stop of the stack.
    ///
    /// Local variables are indexed up from the base pointer.
    #[inline]
    fn load_local(&mut self, index: Index<Local>) -> Result<()> {
        let local = self.value_stack.get_local(self.bp(), index);
        self.value_stack.push(local);
        Ok(())
    }

    /// The [`LoadCapture`][Op::LoadCapture] instruction pulls a value out of
    /// the currently-executing closure's captures and places it on the top of
    /// the stack.
    #[inline]
    fn load_capture(&mut self, index: Index<Capture>) -> Result<()> {
        let value = self
            .current_closure()
            .get_capture_cell(index)
            .contents()
            .get(self.value_stack());

        self.value_stack.push(value);
        Ok(())
    }

    /// The [`DefineLocal`][Op::DefineLocal] instruction increments the top of the
    /// stack by pushing a `()`. This has the effect of leaving the value
    /// previous on the top of the stack around.
    #[inline]
    fn define_local(&mut self) -> Result<()> {
        self.value_stack.push(Value::UNIT);
        Ok(())
    }

    /// The [`LoadClosure`][Op::LoadClosure] instruction creates an instance of
    /// the closure described by the indexed [`Function`] in the current
    /// module, and leaves it on the stack.
    #[inline]
    fn load_closure(&mut self, index: Index<Function>) -> Result<()> {
        let current_closure: Gc<Closure> = self.current_closure();
        let current_module = current_closure.module();

        let prototype = *current_module.get(index).unwrap();
        let new_closure: Gc<Closure> = self.make_from(prototype);
        self.value_stack.push(Value::from(new_closure));

        // now we set up the capture cells
        let capture_count = prototype.capture_count();

        for i in 0..capture_count {
            let capture_index: Index<compiler::Capture> = Index::new(i);

            let capture = prototype.get(capture_index).unwrap();

            let cell = match capture {
                Capture::Local(local_index) => {
                    let index =
                        ValueStack::as_absolute_index(self.bp(), *local_index);
                    let new_cell = self.make_from(index);
                    self.open_captures.push(new_cell);
                    new_cell
                }

                Capture::Recapture(capture_index) => {
                    current_closure.get_capture_cell(*capture_index)
                }
            };

            new_closure.push_capture_cell(cell);
        }

        Ok(())
    }

    /// The [`Call`][Op::Call] instruction calls a function passing the
    /// indicated number of arguments. This is done by creating and pushing a
    /// new frame on the [`CallStack`][crate::call_stack::CallStack].
    ///
    /// The target of the function call is the value before the arguments on
    /// the stack.
    #[inline]
    fn call(&mut self, arg_count: u32) -> Result<()> {
        let bp = self.value_stack.index_from_top(arg_count);

        let target = self
            .value_stack
            .get(bp)
            .expect("call target must be in range on the stack")
            .as_gc::<Closure>()?;

        let parameter_count = target.prototype().parameter_count();

        if parameter_count != arg_count {
            return Err(Error::InvalidArgCount {
                expected: parameter_count,
                found: arg_count,
            });
        }

        let new_frame = CallFrame::new(Index::START, bp);
        self.call_stack.push(new_frame);

        Ok(())
    }

    /// The [`Return`][Op::Return] instruction returns from a function call,
    /// which means it saves the top of the stack, pops the frame, drops the
    /// values up to the old base pointer, and then puts the result back on the
    /// stack.
    #[inline]
    fn r#return(&mut self) -> Result<()> {
        self.close_captures_up_to(self.bp().saturating_next());

        let frame = self.call_stack.pop();
        let result = self.value_stack.last();
        self.value_stack.set(frame.bp, result);
        self.value_stack.truncate_to(frame.bp.next().unwrap());
        Ok(())
    }

    /// The [`List(n)`][Op::List] instruction takes the top `n` values on the
    /// stack and makes them the elements of a new list which is let on the top
    /// of the stack.
    #[inline]
    fn list(&mut self, n: u32) -> Result<()> {
        let slice = self.value_stack().last_n(n as usize);

        debug_assert_eq!(slice.len(), n as usize);

        let vec = Vec::from(slice);
        let list: Gc<List> = self.make_from(vec);
        let value = Value::gc(list);

        if n > 0 {
            self.value_stack.set_from_top(n - 1, value);
            self.value_stack.truncate_by(n - 1);
        } else {
            self.value_stack.push(value);
        }

        Ok(())
    }

    /// The [`Jump(i)`][Op::Jump] instruction jumps to `i` in the current
    /// prototype. We don't have inter-function or inter-module jumps.
    #[inline]
    fn jump(&mut self, i: Index<Op>) -> Result<()> {
        *self.pc_mut() = i;
        Ok(())
    }

    /// The [`BranchFalse(i)`][Op::BranchFalse] instruction consumes teh top of
    /// the stack, and if it [`is_truthy`][PrimitiveOperations::is_truthy] then
    /// continues on. If it's not, then it jumps to `i`.
    #[inline]
    fn branch_false(&mut self, i: Index<Op>) -> Result<()> {
        let truthy = self.value_stack.last().is_truthy();
        self.value_stack.pop();

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
        F: Fn(&Value, &mut VirtualMachine) -> std::result::Result<Value, E>,
        E: Into<Error>,
    {
        let arg = self.value_stack.last();

        let result = op(&arg, self).map_err(Into::into)?;
        self.value_stack.set_from_top(0, result);
        Ok(())
    }

    /// Performs a binary operation `op` which applies some function to the two
    /// values on the top of the stack, replacing them.
    #[inline]
    fn binop<F, E>(&mut self, op: F) -> Result<()>
    where
        F: Fn(
            &Value,
            Value,
            &mut VirtualMachine,
        ) -> std::result::Result<Value, E>,
        E: Into<Error>,
    {
        // The order here is tricky, we don't want to remove the operands from
        // the stack (and the GC root set) until after we have the result of
        // `op`.

        let rhs = self.value_stack.get_from_top(0);
        let lhs = self.value_stack.get_from_top(1);

        let result = op(&lhs, rhs, self).map_err(Into::into)?;

        self.value_stack.set_from_top(1, result);
        self.value_stack.pop();

        Ok(())
    }
}

/// An adapter that makes our comparator operations work more like other binary
/// operators. Just a helper.
#[inline(always)]
fn cmp(
    op: fn(&Value, &Value) -> Option<bool>,
) -> impl Fn(&Value, Value, &mut VirtualMachine) -> std::result::Result<Value, Error>
{
    #[inline(always)]
    move |lhs, rhs, _| {
        op(lhs, &rhs).map(Value::bool).ok_or_else(|| {
            Error::OperationNotSupported {
                type_name: lhs.type_name(),
                op_name: "cmp",
            }
        })
    }
}
