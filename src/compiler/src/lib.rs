//! Build a [`Module`] that the runtime can load and run.

mod constant;
mod debug;
mod function;
mod internal;
mod module;
mod opcode;

pub mod error;

pub use crate::{
    constant::Constant,
    debug::{FunctionDebug, ModuleDebug},
    function::Function,
    internal::{Capture, Local, ModuleBuilder},
    module::Module,
    opcode::Op,
};
