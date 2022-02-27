//! Compiler - Turns syntax trees into modules the runtime can load

mod code;
mod compiler;
mod error;
mod module;

pub mod constant;
pub mod index;
pub mod opcode;
pub mod prototype;

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
    Compiler::new().compile(&syntax)?.build()
}
