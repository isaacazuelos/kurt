//! The virtual machine is the heart of how the language executes code.

use compiler::{Constant, Function, Get, Index, Module, Op};

mod address;
mod call_stack;
mod exit;
mod instructions;
mod open_captures;
mod stack_trace;
mod value_stack;

use crate::classes::{Closure, Keyword, String};
use crate::memory::Gc;
use crate::vm::open_captures::OpenCaptures;
use crate::{memory::GcAny, value::Value, Error, Result};

pub use self::{
    address::Address,
    call_stack::{CallFrame, CallStack},
    exit::Exit,
    value_stack::ValueStack,
};

#[derive(Default)]
/// A struct that manages an instance of the language runtime.
pub struct VirtualMachine {
    // Loaded code
    modules: Vec<Module>,

    // VM
    value_stack: ValueStack,
    call_stack: CallStack,

    // Heap
    pub(crate) heap_head: Option<GcAny>,
    pub(crate) open_captures: OpenCaptures,
}

impl VirtualMachine {
    const MAIN: Index<Module> = Index::new(0);

    pub fn load(&mut self, module: Module) -> Result<()> {
        let new_index = Index::new(self.modules.len() as u32);
        self.modules.push(Module::default());
        self.load_at(module, new_index)
    }

    pub(crate) fn value_stack(&self) -> &ValueStack {
        &self.value_stack
    }

    /// Begin running 'Main.main'
    pub fn start(&mut self) -> Result<Exit> {
        if self.modules.is_empty() {
            return Err(Error::NoMainModule);
        }

        let main_closure: Gc<Closure> =
            self.make_from((VirtualMachine::MAIN, Module::MAIN));

        self.value_stack.push(Value::from(main_closure));

        self.call_stack.push(CallFrame::new(
            Address::new(VirtualMachine::MAIN, Module::MAIN, Index::new(0)),
            Index::new(0),
        ));

        self.run()
    }

    /// Reload the main module specifically.
    pub fn reload_main(&mut self, main: Module) -> Result<()> {
        if self.modules.is_empty() {
            self.modules.push(Module::default());
        }

        self.load_at(main, VirtualMachine::MAIN)
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
        let op = self
            .get(address.module)
            .ok_or(Error::ModuleIndexOutOfRange)?
            .get(address.function)
            .ok_or(Error::PrototypeIndexOutOfRange)?
            .get(address.instruction)
            .ok_or(Error::OpIndexOutOfRange)?;

        Ok(op)
    }
}

impl VirtualMachine {
    fn load_at(&mut self, module: Module, index: Index<Module>) -> Result<()> {
        if self.get(index).is_none() {
            return Err(Error::ModuleIndexOutOfRange);
        }

        self.modules[index.as_usize()] = module;

        Ok(())
    }

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

impl Get<Module> for VirtualMachine {
    fn get(&self, index: Index<Module>) -> Option<&Module> {
        self.modules.get(index.as_usize())
    }
}

impl Get<Function> for VirtualMachine {
    fn get(&self, index: Index<Function>) -> Option<&Function> {
        self.get(self.pc().module)?.get(index)
    }
}
