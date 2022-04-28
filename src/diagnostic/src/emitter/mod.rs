mod code_window;
mod line_art;
mod simple;
mod terminal;

use crate::{Diagnostic, InputCoordinator};

pub use self::{simple::ASCIIEmitter, terminal::FancyEmitter};

/// An [`Emitter`] wraps up the ways you can output diagnostics.
pub trait Emitter {
    /// Emits the diagnostic, presenting it to the user/consumer.
    fn emit(
        &mut self,
        diagnostic: &Diagnostic,
        inputs: &InputCoordinator,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
