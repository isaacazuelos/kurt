//! Debug information, for better error reporting.
//!
//! These are separate so we can keep them optional in the final built modules.

mod function;
mod module;

pub use self::{function::FunctionDebug, module::ModuleDebug};
