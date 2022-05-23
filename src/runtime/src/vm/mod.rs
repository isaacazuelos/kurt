//! The virtual machine is the heart of how the language executes code.

use common::{Get, Index};
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
pub struct VirtualMachine {
    // Loaded code
    modules: Vec<Gc<Module>>,

    // VM
    value_stack: ValueStack,
    call_stack: CallStack,

    // Heap
    pub(crate) heap_head: Option<GcAny>,
    pub(crate) open_captures: OpenCaptures,
}

impl VirtualMachine {
    pub fn new(main: compiler::Module) -> VirtualMachine {
        let mut vm = Self {
            modules: Default::default(),
            call_stack: unsafe { CallStack::new_dangling() },
            value_stack: Default::default(),
            heap_head: Default::default(),
            open_captures: Default::default(),
        };

        let main = vm.make_from(main);
        vm.modules.push(main);
        vm.call_stack = CallStack::new(main);

        vm
    }

    pub fn load(&mut self, module: compiler::Module) -> Result<()> {
        let module = self.make_from(module);
        self.modules.push(module);
        Ok(())
    }

    /// Reload the main module specifically.
    pub fn reload_main(&mut self, main: compiler::Module) -> Result<()> {
        let main = self.make_from(main);

        if self.modules.is_empty() {
            self.modules.push(main);
        } else {
            self.modules[0] = main;
        }

        Ok(())
    }

    /// Begin running 'Main.main'
    pub fn start(&mut self) -> Result<Exit> {
        if self.modules.is_empty() {
            return Err(Error::NoMainModule);
        }

        let main_module = self.modules[0];

        let main_closure: Gc<Closure> =
            self.make_from((main_module, compiler::Module::MAIN));

        self.value_stack.push(Value::from(main_closure));

        self.call_stack.push(CallFrame::new(
            Address::new(main_module, compiler::Module::MAIN, Index::START),
            Index::START,
        ));

        self.run()
    }

    /// Resume the runtime. If it hasn't been started before this will also
    /// start it.
    pub fn resume(&mut self) -> Result<Exit> {
        // TODO: checks.

        // Since we're resuming, the result of the previous computation is on
        // the top of the stack, which we need to clean up.
        self.value_stack.pop();

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
    pub fn pc(&self) -> Address {
        self.call_stack.frame().pc
    }

    /// The program counter is the address of the currently executing piece of
    /// code.
    pub fn pc_mut(&mut self) -> &mut Address {
        &mut self.call_stack.frame_mut().pc
    }

    /// The values on the stack in the current stack frame.
    pub fn stack_frame(&self) -> &[Value] {
        let start = self.bp().as_usize();
        &self.value_stack.as_slice()[start..]
    }

    /// The [`Op`] referred to by the program counter.
    pub(crate) fn op(&self) -> Result<Op> {
        let address = self.pc();
        let module = address.module;

        let op = module
            .get(address.function)
            .ok_or(Error::FunctionIndexOutOfRange)?
            .get(address.instruction)
            .ok_or(Error::OpIndexOutOfRange)?;

        Ok(op)
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
