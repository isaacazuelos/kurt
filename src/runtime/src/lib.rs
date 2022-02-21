//! The language runtime interface.

#![allow(unused)]

mod error;
mod value;

use crate::error::Error;

/// A struct that manages an instance of the language runtime.
#[derive(Default)]
pub struct Runtime {}

impl Runtime {
    /// Attempts to evaluate some input.
    ///
    /// For now 'evaluate' means [`Debug`] pretty print however far into the
    /// pipeline we are, or the [`Debug`] representation for any errors.
    pub fn eval(&mut self, input: &str) {
        fn eval_inner(input: &str) -> Result<(), Error> {
            let module = compiler::compile(input)?;

            println!("{:#?}", module);
            Ok(())
        }

        match eval_inner(input) {
            Ok(()) => {}
            Err(e) => eprintln!("{} [ {:?} ]", e, e),
        }
    }

    /// Print the last 'result' value to standard out.
    ///
    /// This is useful for implementing interactive things. For now it doesn't
    /// show anything meaningful.
    pub fn print(&mut self, prefix: &str) {
        println!("{} <cannot eval>", prefix)
    }
}
