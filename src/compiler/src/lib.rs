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

use syntax::{Module, Parse};

/// Compile in one go.
///
/// If you need to keep the compiler state around for an interactive session,
/// you'll want to look at the documentation for [Compiler].
///
/// # Example
///
/// ```
/// # use compiler::compile;
/// let object = compile(r#" "Hello, world!" "#).unwrap();
/// // Do things with the object here.
/// ```
pub fn compile(input: &str) -> error::Result<Object> {
    let syntax = Module::parse(input)?;
    let mut compiler = Compiler::default();
    compiler.push(&syntax)?;
    compiler.build()
}
