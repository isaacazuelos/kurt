//! Turns syntax trees into [`Object`] values that the runtime can
//! load and run.

mod code;
mod compiler;
mod error;
mod object;

pub mod constant;
pub mod index;
pub mod local;
pub mod opcode;
pub mod prototype;

pub use crate::{compiler::Compiler, error::Error, object::Object};

use diagnostic::Diagnostic;

/// Compile in one go.
///
/// If you need to keep the compiler state around for an interactive session,
/// you'll want to look at the documentation for [Compiler].
///
/// # Example
///
/// ```
/// # use compiler::compile;
/// # use syntax::{Module, Parse};
/// let syntax = Module::parse(r#" "Hello, world!" "#).unwrap();
/// let object = compile(&syntax).unwrap();
/// // Do things with the object here.
/// ```
pub fn compile(syntax: &syntax::Module) -> Result<Object, Diagnostic> {
    let mut compiler = Compiler::default();
    compiler.push(syntax)?;
    let object = compiler.build()?;
    Ok(object)
}
