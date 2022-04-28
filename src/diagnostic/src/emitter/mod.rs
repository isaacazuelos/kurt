mod simple;

use crate::{Diagnostic, InputCoordinator};

pub use self::simple::ASCIIPrinter;

/// An [`Emitter`] wraps up the ways you can output diagnostics.
pub trait Emitter {
    /// Emits the diagnostic, presenting it to the user/consumer.
    fn emit(&mut self, diagnostic: &Diagnostic, inputs: &InputCoordinator);
}
