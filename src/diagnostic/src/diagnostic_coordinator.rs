//! Diagnostic Coordinator handles collecting any diagnostics produced, and
//! emitting them at the right times, and in the right formats.

use crate::{diagnostic::Diagnostic, InputCoordinator};

#[derive(Default)]
pub struct DiagnosticCoordinator {
    /// A sorted collection of all the registered diagnostics.
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCoordinator {
    pub fn register(&mut self, issue: Diagnostic) {
        self.diagnostics.push(issue.into());
    }

    pub fn emit(mut self, inputs: &InputCoordinator) {
        self.diagnostics
            .sort_by_cached_key(|d| (d.input_id(), d.location()));

        for d in &self.diagnostics {
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
}
