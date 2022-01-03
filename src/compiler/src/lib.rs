//! Compiler - Turns syntax trees into modules the runtime can load

mod code;
mod compiler;
mod constant;
mod error;
mod module;
mod opcode;
mod prototype;

pub use crate::{compiler::Compiler, error::Error, module::Module};

use syntax::Parse;

/// Compile a module in one go.
///
/// # Example
///
/// ```
/// # use compiler::compile;
/// let module = compile(r#" "Hello, world!" "#);
/// // do things with the module here
/// ```
pub fn compile(input: &str) -> error::Result<Module> {
    let syntax = syntax::Module::parse(input)?;
    let mut compiler = Compiler::new();

    compiler.module(&syntax)?;
    compiler.build()
}
