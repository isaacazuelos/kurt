//! The virtual machine is the heart of how the language executes code.

use common::Index;
use compiler::{Constant, Op};

mod address;
mod call_stack;
mod exit;
mod instructions;
mod open_captures;
mod stack_trace;
mod value_stack;

use crate::{
    classes::{Closure, Keyword, Module, String},
    memory::{Gc, GcAny},
    value::Value,
    vm::open_captures::OpenCaptures,
    Error, Result,
};

pub use self::{
    address::Address,
    call_stack::{CallFrame, CallStack},
    exit::Exit,
    value_stack::ValueStack,
};

/// A struct that manages an instance of the language runtime.
#[derive(Default)]
pub struct VirtualMachine {
    modules: Vec<Gc<Module>>,

    // VM
    value_stack: ValueStack,
    call_stack: CallStack,

    // Heap
    pub(crate) heap_head: Option<GcAny>,
    pub(crate) open_captures: OpenCaptures,
}

impl VirtualMachine {
    /// Load a module into the runtime and execute its top-level code.
    pub fn load(&mut self, module: compiler::Module) -> Result<Exit> {
        let live_module: Gc<Module> = self.make_from(());

        self.modules.push(live_module);

        unsafe {
            // TODO: there are probably crazy GC issues here
            Module::destructively_set_up_from_compiler_module(
                live_module,
                module,
                self,
            )
        };

        let main_closure: Gc<Closure> = self.make_from(live_module.main());

        let bp = self.value_stack.index_from_top(0);
        self.value_stack.push(Value::from(main_closure));
        let pc = Address::new(main_closure, Index::START);

        self.call_stack.push(CallFrame::new(pc, bp));

        self.run()
    }

    pub fn reload_main(&mut self, _new_main: compiler::Module) -> Result<()> {
        todo!()
    }

    /// Resume the runtime. If it hasn't been started before this will also
    /// start it.
    pub fn resume(&mut self) -> Result<Exit> {
        self.run()
    }

    /// A string containing a representation of the last value on the stack.
    ///
    /// This is useful for the `repl` and `eval` subcommands.
    pub fn last_result(&self) -> std::string::String {
        format!("{:?}", self.value_stack.last())
    }
}

impl VirtualMachine {
    pub(crate) fn value_stack(&self) -> &ValueStack {
        &self.value_stack
    }

    /// The base pointer, or the value which indicates where in the stack values
    /// pertaining to the currently executing closure begin.
    ///
    /// The value below the base pointer is the closure that's currently
    /// executing.
    pub fn bp(&self) -> Index<ValueStack> {
        self.call_stack.frame().bp
    }

    /// The program counter is the address of the currently executing piece of
    /// code.
    pub fn pc(&self) -> &Address {
        self.call_stack.frame().pc()
    }

    /// The program counter is the address of the currently executing piece of
    /// code.
    pub fn pc_mut(&mut self) -> &mut Address {
        self.call_stack.frame_mut().pc_mut()
    }

    /// The values on the stack in the current stack frame.
    pub fn stack_frame(&self) -> &[Value] {
        let start = self.bp().as_usize();
        &self.value_stack.as_slice()[start..]
    }

    /// The [`Op`] referred to by the program counter.
    pub(crate) fn op(&self) -> Result<Op> {
        let address = self.pc();

        address
            .closure
            .get_op(address.instruction)
            .ok_or(Error::OpIndexOutOfRange)
    }
}

impl VirtualMachine {
    /// Inflate a [`Constant`] into a full-fledged runtime value.
    fn inflate(&mut self, constant: &Constant) -> Result<Value> {
        match constant {
            Constant::Character(c) => Ok(Value::char(*c)),
            Constant::Float(bits) => Ok(Value::float(f64::from_bits(*bits))),
            Constant::String(s) => {
                let string: Gc<String> = self.make_from(s.as_str());
                Ok(Value::gc(string))
            }
            Constant::Keyword(kw) => {
                let keyword: Gc<Keyword> = self.make_from(kw.as_str());
                Ok(Value::gc(keyword))
            }
        }
    }
}
