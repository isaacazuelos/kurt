//! The language runtime interface.

#![allow(unused)]

mod error;
mod module;
mod value;

use compiler::{constant::Constant, Module as ModuleDescription};
use module::Module;
use value::Value;

use crate::error::{Error, Result};

/// A struct that manages an instance of the language runtime.
#[derive(Debug, Default)]
pub struct Runtime {
    main: Module,
    stack: Vec<Value>,
}

impl Runtime {
    /// Attempts to evaluate some input.
    ///
    /// For now 'evaluate' means [`Debug`] pretty print however far into the
    /// pipeline we are, or the [`Debug`] representation for any errors.
    pub fn eval(&mut self, input: &str) {
        match self.eval_inner(input) {
            Ok(()) => {}
            Err(e) => eprintln!("{} [ {:?} ]", e, e),
        }
    }

    /// Print the top of the stack to standard out.
    ///
    /// This is useful for implementing interactive things. For now it doesn't
    /// show anything meaningful.
    pub fn print(&mut self, prefix: &str) {
        match self.stack.last() {
            None => print!("()"),
            Some(v) => print!("{:?}", v),
        }
    }

    /// A helper for [`Runtime::eval`] but returning a [`Result`].
    fn eval_inner(&mut self, input: &str) -> Result<()> {
        let object = compiler::compile(input)?;

        self.load(object)?;
        self.run()
    }
}

impl Runtime {
    /// Load a module. For now, we only support one module, `main`, so every
    /// time something loads it replaces it.
    fn load(&mut self, description: ModuleDescription) -> Result<()> {
        let mut constants = Vec::with_capacity(description.constants().len());

        for constant in description.constants() {
            let value = self.inflate(constant)?;
            constants.push(value);
        }

        let module = Module {
            constants,
            main: description.get_main().clone(),
            prototypes: description.get_prototypes().to_owned(),
        };

        self.main = module;
        Ok(())
    }

    fn inflate(&mut self, constant: &Constant) -> Result<Value> {
        match constant {
            Constant::Character(c) => Ok(Value::char(*c)),
            Constant::Number(n) => Value::nat(*n).ok_or(Error::NumberTooBig),
            Constant::Float(bits) => Ok(Value::float(f64::from_bits(*bits))),
            Constant::String(_) => todo!("types which alloc not yet supported"),
        }
    }

    fn run(&mut self) -> Result<()> {
        println!("running with {:#?}", self);
        Ok(())
    }
}
