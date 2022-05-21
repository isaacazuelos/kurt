//! The language runtime interface.

mod error;
mod memory;
mod primitives;
mod value;
mod vm;

#[cfg(feature = "trace")]
mod tracing;

pub use crate::{
    error::{Error, Result},
    value::Value,
    vm::{Exit, VirtualMachine},
};
