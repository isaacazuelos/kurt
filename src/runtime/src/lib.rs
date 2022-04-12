//! The language runtime interface.

mod address;
mod call_stack;
mod error;
mod machine;
mod memory;
mod module;
mod stack;
mod tracing;
mod value;

use address::Address;
use call_stack::{CallFrame, CallStack};
use compiler::{
    self,
    constant::Constant,
    index::{Get, Index},
    opcode::Op,
    prototype::Prototype,
    Object,
};
use memory::closure::Closure;

use crate::{
    memory::{keyword::Keyword, string::String, Gc},
    module::Module,
    stack::Stack,
    value::Value,
};

pub use crate::error::{Error, Result};

/// Each [`Exit`] is a reason a [`Runtime`] may have stopped running (which
/// isn't an [`Error`]).
pub enum Exit {
    /// The runtime hit the end of it's code.
    Halt,

    /// The runtime hit a yield point, which for now means the end of repl code.
    Yield,
}

/// A struct that manages an instance of the language runtime.
#[derive(Default, Debug)]
pub struct Runtime {
    tracing: bool,

    // Loaded code
    modules: Vec<Module>,

    // VM
    stack: Stack,
    call_stack: CallStack,

    // Heap
    heap_head: Option<Gc>,
}

impl Runtime {
    /// Set the Runtime's tracing mode.
    ///
    /// When tracing is on (`true`) the runtime will print some extra
    /// information.
    pub fn set_tracing(&mut self, tracing: bool) {
        self.tracing = tracing;
    }

    /// Is the runtime's tracing mode on?
    pub fn tracing(&self) -> bool {
        self.tracing
    }
}

impl Runtime {
    /// Create a new runtime with `object` as it's main module.
    pub fn new() -> Runtime {
        Runtime {
            tracing: false,

            modules: Vec::new(),

            stack: Stack::default(),
            call_stack: CallStack::default(),

            heap_head: None,
        }
    }

    pub fn load(&mut self, object: Object) -> Result<()> {
        let module = self.make_module(object)?;
        self.modules.push(module);
        Ok(())
    }

    /// Begin running 'Main.main'
    pub fn start(&mut self) -> Result<Exit> {
        // TODO: lots of clean up needed.

        // We only have one module support right now.
        let main_module_index = Index::new(0);
        let main_module = self.modules.first().ok_or(Error::NoMainModule)?;
        let main_prototype_index =
            Index::new((main_module.prototypes.len() - 1) as u32);

        let indexes = (main_module_index, main_prototype_index);
        let main_closure = self.make_from::<Closure, _>(indexes);

        self.stack.push(Value::object(main_closure));

        self.call_stack.push(CallFrame::new(
            Address::new(main_module_index, main_prototype_index, Index::START),
            self.stack.index_from_top(0),
        ));

        // TODO: checks.
        self.run()
    }

    /// Reload the main module specifically.
    pub fn reload_main(&mut self, main: Object) -> Result<()> {
        // TODO: We should replace this with a generic `reload`.
        // TODO: We should do some sanity checks here.
        let new = self.make_module(main)?;
        self.modules[0] = new;
        Ok(())
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
    fn make_module(&mut self, object: Object) -> Result<Module> {
        let mut constants = Vec::with_capacity(object.constants().len());

        for constant in object.constants() {
            let value = self.inflate(constant)?;
            constants.push(value);
        }

        Ok(Module {
            constants,
            prototypes: object.prototypes().to_owned(),
        })
    }

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
        self.get(address.module)
            .ok_or(Error::ModuleIndexOutOfRange)?
            .get(address.prototype)
            .ok_or(Error::PrototypeIndexOutOfRange)?
            .get(address.instruction)
            .ok_or(Error::OpIndexOutOfRange)
            .cloned()
    }
}

impl Runtime {
    /// Inflate a [`Constant`] into a full-fledged runtime value.
    fn inflate(&mut self, constant: &Constant) -> Result<Value> {
        match constant {
            Constant::Character(c) => Ok(Value::char(*c)),
            Constant::Number(n) => Value::nat(*n).ok_or(Error::NumberTooBig),
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

impl Get<Prototype> for Runtime {
    fn get(&self, index: Index<Prototype>) -> Option<&Prototype> {
        self.get(self.pc().module)?.get(index)
    }
}
