//! Compiler - Turns syntax trees into objects the runtime can load.
//!
//! Each object is compiled completely independently, to be linked together by
//! the runtime when loaded.
//!
//! Once you're ready to produce an [`Object`], you can do so by calling
//! [`Compiler::build`].

mod capture;
mod code;
mod code_gen;
mod function;
mod local;
mod module;
mod pool;

pub(crate) use self::{function::FunctionBuilder, pool::ConstantPool};

pub use self::{capture::Capture, local::Local, module::ModuleBuilder};
