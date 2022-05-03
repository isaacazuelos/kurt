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
