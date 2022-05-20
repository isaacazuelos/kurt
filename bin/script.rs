//! Run some input as a script.

use std::{fs::File, io::Read, path::PathBuf};

use compiler::Module;
use diagnostic::{
    verify_utf8, Diagnostic, DiagnosticCoordinator, InputCoordinator,
};
use runtime::Runtime;

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

        let main = match Module::try_from(input) {
            Ok(object) => object,
            Err(d) => {
                diagnostics.register(d.input(id));
                diagnostics.emit(&inputs);
                return;
            }
        };

        if args.dump {
            println!("{:#?}", main);
            return;
        }

        let mut runtime = Runtime::new();

        match runtime.load(main) {
            Ok(object) => object,
            Err(e) => {
                let d = Diagnostic::new(format!("{e}"));
                diagnostics.register(d);
                diagnostics.emit(&inputs);
                return;
            }
        };

        if let Err(e) = runtime.start() {
            let d = Diagnostic::new(format!("{e}"));
            diagnostics.register(d);
            diagnostics.emit(&inputs);
        }
    }
}
