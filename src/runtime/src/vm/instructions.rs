//! The virtual machine's big dispatch loop

use common::{i48, Get, Index};
use compiler::{Capture, Constant, Local, Op};

use crate::{
    classes::{Function, List},
    error::Result,
    memory::Gc,
    primitives::PrimitiveOperations,
    value::Value,
    vm::{stack::StackTop, CallFrame, Exit, Stack},
    Error, VirtualMachine,
};

impl VirtualMachine {
    /// Start the VM up again.
    pub(crate) fn run(&mut self) -> Result<Exit> {
        loop {
            #[cfg(feature = "trace")]
            self.trace();

            match self.fetch() {
                // control
                Op::Halt => return self.halt(),
                Op::Nop => continue,

                // stack
                Op::Dup => self.dup()?,
                Op::Pop => self.stack.pop(),
                Op::Close(n) => self.close(n)?,

                // values
                Op::True => self.stack.push(Value::TRUE),
                Op::False => self.stack.push(Value::FALSE),
                Op::Unit => self.stack.push(Value::UNIT),
                Op::U48(n) => self.stack.push(Value::int(
                    i48::try_from(n).map_err(|_| Error::NumberTooBig)?,
                )),
                Op::I48(n) => self.stack.push(Value::from(n)),

                Op::LoadSelf => self.load_self()?,
                Op::LoadConstant(i) => self.load_constant(i)?,
                Op::LoadLocal(i) => self.load_local(i)?,
                Op::LoadCapture(i) => self.load_capture(i)?,
                Op::LoadFunction(i) => self.load_function(i)?,
                Op::DefineLocal => self.define_local()?,
                Op::Index => self.binop(Value::index)?,

                // Assignment
                Op::SetLocal(i) => self.set_local(i)?,
                Op::SetIndex => self.set_index()?,

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

    /// Get the op at the program counter, and then increments the counter.
    #[inline]
    fn fetch(&mut self) -> Op {
        let op = self.op();
        self.pc_mut().saturating_increment();
        op
    }

    /// Closes any open upvalues which occur in the open list with a stack index
    /// above `top`.
    #[inline]
    fn close_captures_above(&mut self, top: Index<Stack>) {
        while let Some(cell) = self.open_captures.pop_if_above(top) {
            let index = cell
                .stack_index()
                .expect("cells in the open list should be open, and therefor have a stack index");
            let value = self.stack[index];
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

    /// The [`Dup`][Op::Dup] instruction duplicates the value on the top of the
    /// stack. Does nothing if the stack is empty.
    #[inline]
    fn dup(&mut self) -> Result<()> {
        if let Some(value) = self.stack().last().cloned() {
            self.stack.push(value);
        }

        Ok(())
    }

    /// The [`Close`][Op::Close] instructions slides the value on the top of the
    /// stack back `n` spaces, and closes any open captures along the way.
    #[inline]
    fn close(&mut self, n: u32) -> Result<()> {
        let new_top: Index<Stack> = self.stack.from_top(Index::new(n));
        self.close_captures_above(new_top);

        let kept = self
            .stack()
            .last()
            .expect("when executing Close, the stack cannot be empty");

        self.stack[new_top] = *kept;
        self.stack.truncate_above(new_top);

        Ok(())
    }

    /// The [`LoadSelf`][Op::LoadSelf] instruction loads the currently-executing
    /// closure (at the base pointer) and places a copy on the stop of the
    /// stack.
    #[inline]
    fn load_self(&mut self) -> Result<()> {
        let value = Value::from(self.current_closure());
        self.stack.push(value);
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
            .constant(index)
            .expect("constant index out of range");

        self.stack.push(constant);
        Ok(())
    }

    /// The [`LoadLocal`][Op::LoadLocal] instruction loads a copies the [`Value`]
    /// of a local variable places it on the stop of the stack.
    ///
    /// Local variables are indexed up from the base pointer.
    #[inline]
    fn load_local(&mut self, index: Index<Local>) -> Result<()> {
        let local = self.stack[(self.bp(), index)];
        self.stack.push(local);
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
            .get(self.stack());

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

    /// The [`LoadFunction`][Op::LoadFunction] instruction creates an instance of
    /// the closure described by the indexed [`Function`] in the current
    /// module, and leaves it on the stack.
    #[inline]
    fn load_function(
        &mut self,
        index: Index<compiler::Function>,
    ) -> Result<()> {
        let current_closure: Gc<Function> = self.current_closure();
        let current_module = current_closure.module();

        let prototype = *current_module.get(index).unwrap();
        let new_closure: Gc<Function> = self.make_from(prototype);
        self.stack.push(Value::from(new_closure));

        // now we set up the capture cells
        let capture_count = prototype.capture_count();

        for i in 0..capture_count {
            let capture_index: Index<compiler::Capture> = Index::new(i);

            let capture = prototype.get(capture_index).unwrap();

            let cell = match capture {
                Capture::Local(local_index) => {
                    let index = Stack::from_local(self.bp(), *local_index);
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

    /// The [`SetLocal`][Op::SetLocal] instruction sets the local binding at the
    /// given index to the value on the top of the stack. This leaves the new
    /// value on the stack.
    fn set_local(&mut self, index: Index<Local>) -> Result<()> {
        let new_value = *self.stack.last().expect(
            "SetLocal expects the new value on the stack, but it was empty",
        );
        let bp = self.bp();
        self.stack[(bp, index)] = new_value;
        Ok(())
    }

    /// The [`SetIndex`][Op::SetIndex] instruction needs 3 values on the stack,
    /// which (from the top) are the new value, the key to index with, and the
    /// target value.
    fn set_index(&mut self) -> Result<()> {
        let new = self.stack[Index::<StackTop>::new(0)];
        let key = self.stack[Index::<StackTop>::new(1)];

        let col_index = self.stack.from_top(Index::new(2));
        let col = self.stack[col_index];

        col.set_index(key, new, self)?;

        self.stack[col_index] = new;
        self.stack.pop();
        self.stack.pop();
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
        let bp = self.stack.from_top(Index::new(arg_count));
        let target: Gc<Function> = self.stack[bp].try_into()?;
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
        self.close_captures_above(self.bp().saturating_previous());

        let frame = self.call_stack.pop();
        let result = self.stack.last().expect("return on empty stack");
        self.stack[frame.bp()] = *result;

        self.stack.truncate_above(frame.bp());
        Ok(())
    }

    /// The [`List(n)`][Op::List] instruction takes the top `len` values on the
    /// stack and makes them the elements of a new list which is left on top of
    /// the stack.
    #[inline]
    fn list(&mut self, len: u32) -> Result<()> {
        // Empty lists don't have on-stack values to pop, so we have to do
        // things a bit differently.
        if len == 0 {
            let empty_list: Gc<List> = self.make_from(Vec::new());
            self.stack.push(Value::from(empty_list));
            return Ok(());
        }

        let under_elements = Index::<StackTop>::new(len);

        let value = {
            let values = self.stack().above(under_elements).to_vec();
            debug_assert_eq!(values.len(), len as usize);
            let list: Gc<List> = self.make_from(values);
            Value::from(list)
        };

        // We already handled empty lists, so there's a first element.
        let first_element =
            self.stack.from_top(under_elements.saturating_previous());
        self.stack[first_element] = value;
        self.stack.truncate_above(first_element);

        Ok(())
    }

    /// The [`Jump(i)`][Op::Jump] instruction jumps to `i` in the current
    /// prototype. We don't have inter-function or inter-module jumps.
    #[inline]
    fn jump(&mut self, i: Index<Op>) -> Result<()> {
        *self.pc_mut() = i;
        Ok(())
    }

    /// The [`BranchFalse(i)`][Op::BranchFalse] instruction consumes the top of
    /// the stack, and if it [`is_truthy`][PrimitiveOperations::is_truthy] then
    /// continues on. If it's not, then it jumps to `i`.
    #[inline]
    fn branch_false(&mut self, i: Index<Op>) -> Result<()> {
        let truthy = self
            .stack
            .last()
            .expect("the stack should not be empty when executing BranchFalse")
            .is_truthy();

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
        F: Fn(&Value, &mut VirtualMachine) -> std::result::Result<Value, E>,
        E: Into<Error>,
    {
        let arg = *self
            .stack
            .last()
            .expect("unary operator expected a value on the stack");

        let result = op(&arg, self).map_err(Into::into)?;
        self.stack[Index::<StackTop>::START] = result;
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

        let top = Index::<StackTop>::new(0);
        let one_below = Index::<StackTop>::new(1);

        let rhs = self.stack[top];
        let lhs = self.stack[one_below];

        let result = op(&lhs, rhs, self).map_err(Into::into)?;

        self.stack[one_below] = result;
        self.stack.pop();

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
