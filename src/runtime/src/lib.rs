//! The language runtime interface.

use compiler;
use syntax;

mod error;
mod value;

use crate::error::Error;

/// A struct that manages an instance of the language runtime.
pub struct Runtime {}

impl Default for Runtime {
    fn default() -> Runtime {
        Runtime {}
    }
}

impl Runtime {
    /// Attempts to evaluate some input.
    ///
    /// For now 'evaluate' means [`Debug`] pretty print however far into the
    /// pipeline we are, or the [`Debug`] representation for any errors.
    pub fn eval(&mut self, input: &[u8]) {
        fn eval_inner(input: &[u8]) -> Result<(), Error> {
            let input = syntax::verify_utf8(input)?;
            let module = compiler::compile(&input)?;

            println!("{:#?}", module);
            Ok(())
        }

        match eval_inner(input) {
            Ok(()) => {}
            Err(e) => eprintln!("{} [ {:?} ]", e, e),
        }
    }
}
