//! The language runtime interface.

mod address;
mod error;
mod machine;
mod memory;
mod module;
mod value;

use address::Address;
use compiler::{
    self, constant::Constant, index::Indexable, opcode::Op,
    prototype::Prototype,
};

use crate::{
    memory::{string::String, GcObj},
    module::Module,
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
#[derive(Debug, Default)]
pub struct Runtime {
    main: Module, // TODO: make this a vec with indexes that are linked.
    pc: Address,
    stack: Vec<Value>,
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
        if let Some(last) = self.stack.last() {
            // FIXME: This is almost certainly a bad idea for the GC.
            *last
        } else {
            Value::UNIT
        }
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
        let index = self.pc.prototype;
        self.current_module()?.prototype(index)
    }

    /// The currently-executing opcode.
    fn current_op(&self) -> Result<Op> {
        let index = self.pc.instruction;
        self.current_prototype()?
            .get(index)
            .cloned()
            .ok_or(Error::OpIndexOutOfRange)
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
                let string: GcObj = self.make_from::<String, _>(s.as_str());
                Ok(Value::object(string))
            }
        }
    }
}
