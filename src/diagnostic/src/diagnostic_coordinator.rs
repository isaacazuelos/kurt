//! Diagnostic Coordinator handles collecting any diagnostics produced, and
//! emitting them at the right times, and in the right formats.

use crate::emitter::{ASCIIPrinter, Emitter};
use crate::{diagnostic::Diagnostic, InputCoordinator};

pub struct DiagnosticCoordinator {
    /// A sorted collection of all the registered diagnostics.
    diagnostics: Vec<Diagnostic>,

    /// The emitter that will be used to present the diagnostics.
    emitter: Box<dyn Emitter>,
}

impl Default for DiagnosticCoordinator {
    fn default() -> Self {
        DiagnosticCoordinator {
            diagnostics: Vec::new(),
            emitter: Box::new(ASCIIPrinter::default()),
        }
    }
}

impl DiagnosticCoordinator {
    pub fn register(&mut self, issue: Diagnostic) {
        self.diagnostics.push(issue);
    }

    pub fn emit(&mut self, inputs: &InputCoordinator) {
        self.diagnostics
            .sort_by_cached_key(|d| (d.input_id(), d.location()));

        for d in &self.diagnostics {
            self.emitter.emit(d, inputs);
        }
    }

    pub fn clear(&mut self) {
        self.diagnostics.clear()
    }
}
