//! The language runtime interface.

mod address;
mod call_stack;
mod error;
mod machine;
mod memory;
mod primitives;
mod stack;
mod stack_trace;
mod value;

#[cfg(feature = "trace")]
mod tracing;

use address::Address;
use call_stack::{CallFrame, CallStack};
use compiler::{Constant, Function, Get, Index, Module, Op};
use memory::closure::Closure;

use crate::value::i48_type::i48;

use crate::{
    memory::{keyword::Keyword, string::String, Gc},
    stack::Stack,
    value::Value,
};

pub use crate::error::{Error, Result};

/// Each [`Exit`] is a reason a [`Runtime`] may have stopped running (which
/// isn't an [`Error`]).
#[derive(Debug)]
pub enum Exit {
    /// The runtime hit the end of it's code.
    Halt,

    /// The runtime hit a yield point, which for now means the end of repl code.
    Yield,
}

/// A struct that manages an instance of the language runtime.
#[derive(Default, Debug)]
pub struct Runtime {
    // Loaded code
    modules: Vec<Module>,

    // VM
    stack: Stack,
    call_stack: CallStack,

    // Heap
    heap_head: Option<Gc>,
    open_captures: Vec<Value>,
}

impl Runtime {
    const MAIN: Index<Module> = Index::new(0);

    /// Create a new runtime with `object` as it's main module.
    pub fn new() -> Runtime {
        Runtime {
            modules: Vec::new(),

            stack: Stack::default(),
            call_stack: CallStack::default(),

            heap_head: None,
            open_captures: Vec::new(),
        }
    }

    pub fn load(&mut self, module: Module) -> Result<()> {
        let new_index = Index::new(self.modules.len() as u32);
        self.modules.push(Module::default());
        self.load_at(module, new_index)
    }

    /// Begin running 'Main.main'
    pub fn start(&mut self) -> Result<Exit> {
        if self.modules.is_empty() {
            return Err(Error::NoMainModule);
        }

        let main_closure =
            self.make_from::<Closure, _>((Runtime::MAIN, Module::MAIN));

        self.stack.push(Value::object(main_closure));

        self.call_stack.push(CallFrame::new(
            Address::new(Runtime::MAIN, Module::MAIN, Index::new(0)),
            self.stack.index_from_top(0),
        ));

        self.run()
    }

    /// Reload the main module specifically.
    pub fn reload_main(&mut self, main: Module) -> Result<()> {
        if self.modules.is_empty() {
            self.modules.push(Module::default());
        }

        self.load_at(main, Runtime::MAIN)
    }

    /// Resume the runtime. If it hasn't been started before this will also
    /// start it.
    pub fn resume(&mut self) -> Result<Exit> {
        // TODO: checks.

        // Since we're resuming, the result of the previous computation is on
        // the top of the stack, which we need to clean up.
        self.stack.pop();

        self.run()
    }

    /// A string containing a representation of the last value on the stack.
    ///
    /// This is useful for the `repl` and `eval` subcommands.
    pub fn last_result(&self) -> std::string::String {
        format!("{:?}", self.stack.last())
    }
}

impl Runtime {
    /// The base pointer, or the value which indicates where in the stack values
    /// pertaining to the currently executing closure begin.
    ///
    /// The value below the base pointer is the closure that's currently
    /// executing.
    pub fn bp(&self) -> Index<Stack> {
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

    /// The values on the stack after the current stack frame.
    pub fn stack_frame(&self) -> &[Value] {
        let start = self.bp().as_usize();
        &self.stack.as_slice()[start..]
    }

    /// The [`Op`] referred to by the program counter.
    fn op(&self) -> Result<Op> {
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

impl Runtime {
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
            Constant::Number(n) => {
                // TODO: this is wrong at n > i64::MAX, and inelegant.
                //
                // For now everything loads as an integer.
                let i = i48::from_i64(*n as i64).ok_or(Error::NumberTooBig)?;
                Ok(Value::int(i))
            }
            Constant::Float(bits) => Ok(Value::float(f64::from_bits(*bits))),
            Constant::String(s) => {
                let string: Gc = self.make_from::<String, _>(s.as_str());
                Ok(Value::object(string))
            }
            Constant::Keyword(kw) => {
                let keyword = self.make_from::<Keyword, _>(kw.as_str());
                Ok(Value::object(keyword))
            }
        }
    }
}

impl Get<Module> for Runtime {
    fn get(&self, index: Index<Module>) -> Option<&Module> {
        self.modules.get(index.as_usize())
    }
}

impl Get<Function> for Runtime {
    fn get(&self, index: Index<Function>) -> Option<&Function> {
        self.get(self.pc().module)?.get(index)
    }
}
