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
    prototype::Prototype,
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
    /// The runtime hit the end of the main module.
    Halt,
}

/// A struct that manages an instance of the language runtime.
#[derive(Default, Debug)]
pub struct Runtime {
    tracing: bool,

    // Loaded code
    main: Module, // TODO: make this a vec with indexes that are linked.

    // VM
    stack: Stack,
    call_stack: CallStack,

    // Heap
    heap_head: Option<Gc>,
    interned_constants: Vec<Gc>,
}

impl Runtime {
    /// Set the runtime's tracing.
    pub fn set_tracing(&mut self, tracing: bool) {
        self.tracing = tracing;
    }
}

impl Runtime {
    /// Attempts to evaluate some input.
    ///
    /// For now 'evaluate' means [`Debug`] pretty print however far into the
    /// pipeline we are, or the [`Debug`] representation for any errors.
    pub fn eval(&mut self, input: &str) -> Result<Exit> {
        let object = compiler::compile(input)?;

        self.reload_main(object)?;
        self.run()
    }

    // A [`Display`][std::fmt::Display]-able view of the last result left on the
    // stack. This is useful for the `repl` and `eval` subcommands.
    pub fn last_result(&self) -> Value {
        self.stack.last()
    }

    /// A helper for [`Runtime::eval`] but returning a [`Result`].
    ///
    /// Reload the main module specifically.
    pub fn reload_main(&mut self, module: compiler::Module) -> Result<()> {
        let mut constants = Vec::with_capacity(module.constants().len());

        for constant in module.constants() {
            let value = self.inflate(constant)?;
            constants.push(value);
        }

        // TODO: We could do some verification here that what we're reloading
        //       looks sane.

        self.main = Module {
            constants,
            prototypes: module.prototypes().to_owned(),
        };

        Ok(())
    }

    /// Resume the runtime. If it hasn't been started before this will also
    /// start it.
    pub fn resume(&mut self) -> Result<Exit> {
        // TODO: sanity checks.
        self.run()
    }

    /// The module which contains the currently-executing code.
    fn current_module(&self) -> Result<&Module> {
        Ok(&self.main)
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
