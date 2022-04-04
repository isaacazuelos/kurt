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

use call_stack::{CallFrame, CallStack};
use compiler::{
    self, constant::Constant, index::Indexable, opcode::Op,
    prototype::Prototype, Object,
};

use crate::{
    memory::{string::String, Gc},
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
    interned_constants: Vec<Gc>,
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
    pub fn new(object: Object) -> Result<Runtime> {
        let mut rt = Runtime {
            tracing: false,

            modules: Vec::new(),

            stack: Stack::default(),
            call_stack: CallStack::default(),

            heap_head: None,
            interned_constants: Vec::new(),
        };

        let main = rt.make_module(object)?;
        rt.modules.push(main);

        Ok(rt)
    }

    /// A string containing a representation of the last value on the stack.
    ///
    /// This is useful for the `repl` and `eval` subcommands.
    pub fn last_result(&self) -> std::string::String {
        format!("{:?}", self.stack.last())
    }

    pub fn start(&mut self) -> Result<Exit> {
        // TODO: checks.
        self.run()
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

    /// Reload the main module specifically.
    pub fn reload_main(&mut self, main: Object) -> Result<()> {
        // TODO: We should replace this with a generic `reload`.
        // TODO: We should do some sanity checks here.
        let new = self.make_module(main)?;
        self.modules[0] = new;
        Ok(())
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
            main: object.main().clone(),
            constants,
            prototypes: object.prototypes().to_owned(),
        })
    }

    /// The module which contains the currently-executing code.
    fn current_module(&self) -> Result<&Module> {
        let index = self.current_frame().pc.module;
        self.modules
            .get(index.as_usize())
            .ok_or(Error::ModuleIndexOutOfRange)
    }

    /// The currently-executing prototype.
    fn current_prototype(&self) -> Result<&Prototype> {
        let index = self.current_frame().pc.prototype;
        self.current_module()?.prototype(index)
    }

    /// The currently-executing opcode.
    pub(crate) fn current_op(&self) -> Result<Op> {
        let index = self.current_frame().pc.instruction;
        self.current_prototype()?
            .get(index)
            .cloned()
            .ok_or(Error::OpIndexOutOfRange)
    }

    /// The current call frame.
    fn current_frame(&self) -> &CallFrame {
        self.call_stack.frame()
    }

    /// The values on the stack after the current stack frame.
    fn current_stack(&self) -> &[Value] {
        &self.stack.as_slice()[self.current_frame().bp.as_usize()..]
    }

    /// The current call frame.
    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.call_stack.frame_mut()
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
                self.interned_constants.push(string);
                Ok(Value::object(string))
            }
        }
    }
}
