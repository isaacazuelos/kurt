//! Turns syntax trees into [`Object`] values that the runtime can
//! load and run.

mod constant;
mod function;
mod index;
mod internal;
mod module;
mod opcode;

pub mod error;

pub use crate::{
    constant::Constant,
    function::Function,
    index::{Get, Index},
    internal::{Capture, Local, ModuleBuilder},
    module::Module,
    opcode::Op,
};
