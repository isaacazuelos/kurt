//! Run some input as a script.

use std::{fs::File, io::Read, path::PathBuf};

use compiler::ModuleBuilder;
use diagnostic::{
    verify_utf8, Diagnostic, DiagnosticCoordinator, InputCoordinator,
};
use runtime::VirtualMachine;

use crate::Args;

/// Run a file as a script.
#[derive(clap::Parser)]
pub struct Script {
    filename: PathBuf,
}

impl Script {
    /// Run the file `filename` as a script.
    pub(crate) fn run(&self, args: &Args) {
        let mut inputs = InputCoordinator::default();
        let mut diagnostics = DiagnosticCoordinator::default();

        let mut bytes = Vec::new();

        if let Err(e) = File::open(&self.filename)
            .and_then(|mut file| file.read_to_end(&mut bytes))
        {
            let d = Diagnostic::new(format!("{e}"));
            diagnostics.register(d);
            diagnostics.emit(&inputs);
            return;
        }

        let input = match verify_utf8(&bytes) {
            Ok(s) => s,
            Err(d) => {
                diagnostics.register(d);
                diagnostics.emit(&inputs);
                return;
            }
        };

        let id = inputs.file_input(input.into(), self.filename.clone());

        let main = match ModuleBuilder::default().input(input) {
            Ok(builder) => builder.with_id(Some(id)).build(),

            Err(d) => {
                diagnostics.register(d.input(id));
                diagnostics.emit(&inputs);
                return;
            }
        };

        if args.dump {
            println!("{}", main);
            return;
        }

        let mut runtime = VirtualMachine::new();

        if let Err(e) = runtime.load(main) {
            let d = Diagnostic::new(format!("{e}"));
            diagnostics.register(d);
            diagnostics.emit(&inputs);
            return;
        };

        if let Err(e) = runtime.start() {
            runtime.stack_trace(e, &mut diagnostics);
            diagnostics.emit(&inputs);
        }
    }
}
