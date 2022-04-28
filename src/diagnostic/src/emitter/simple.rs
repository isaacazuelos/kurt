//! A simple, safe, ascii-focused plain-text emitter that writes to stdout.
//!
//! This should be a safe fall-back when we don't know what the output device
//! looks like.

use super::Emitter;
use crate::input_coordinator::InputCoordinator;
use crate::Diagnostic;

#[derive(Default)]
pub struct ASCIIEmitter;

impl ASCIIEmitter {}

impl Emitter for ASCIIEmitter {
    fn emit(
        &mut self,
        d: &Diagnostic,
        inputs: &InputCoordinator,
    ) -> Result<(), Box<dyn std::error::Error>> {
        eprint!("{}", d.message().level());

        let name = d.input_id().map(|id| inputs.get_input_name(id));

        match (name, d.location()) {
            (None, None) => eprint!(": "),
            (None, Some(l)) => eprint!(" {l}:"),
            (Some(n), None) => eprint!(": {n} - "),
            (Some(n), Some(l)) => eprint!(": {n}:{l} - "),
        }

        eprintln!("{}", d.message().text());

        Ok(())
    }
}
