//! A simple, safe, ascii-focused plain-text emitter that writes to stdout.
//!
//! This should be a safe fall-back when we don't know what the output device
//! looks like.

use super::Emitter;
use crate::input_coordinator::InputCoordinator;
use crate::Diagnostic;

#[derive(Default)]
pub struct ASCIIPrinter;

impl ASCIIPrinter {}

impl Emitter for ASCIIPrinter {
    fn emit(&mut self, d: &Diagnostic, inputs: &InputCoordinator) {
        eprint!("{}", d.message().level());

        let name = d.input_id().and_then(|id| inputs.get_input_name(id));

        match (name, d.location()) {
            (None, None) => eprint!(": "),
            (None, Some(l)) => eprint!(" {l}:"),
            (Some(n), None) => eprint!(": {n} - "),
            (Some(n), Some(l)) => eprint!(": {n}:{l} - "),
        }

        eprintln!("{}", d.message().text())
    }
}
