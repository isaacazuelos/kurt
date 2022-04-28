//! Run some input as a script.

use std::{fs::File, io::Read, path::PathBuf};

use compiler::Compiler;
use diagnostic::{Diagnostic, DiagnosticCoordinator, InputCoordinator};
use runtime::Runtime;
use syntax::{Module, Parse};

use crate::Args;

/// Run a file as a script.
#[derive(clap::Parser)]
pub struct Script {
    filename: PathBuf,
}

impl Script {
    /// Run the file `filename` as a script.
    pub(crate) fn run(&self, args: &Args) {
        let inputs = InputCoordinator::default();
        let mut diagnostics = DiagnosticCoordinator::default();
        let mut compiler = Compiler::default();

        let mut input = String::new();

        if let Err(e) = File::open(&self.filename)
            .and_then(|mut file| file.read_to_string(&mut input))
        {
            let d = Diagnostic::new(format!("{e}"));
            diagnostics.register(d);
            diagnostics.emit(&inputs);
            return;
        }

        let syntax = match Module::parse(&input) {
            Ok(object) => object,
            Err(e) => {
                let d = Diagnostic::new(format!("{e}"));
                diagnostics.register(d);
                diagnostics.emit(&inputs);
                return;
            }
        };

        match compiler.push(&syntax) {
            Ok(()) => {}
            Err(e) => {
                let d = e.into();
                diagnostics.register(d);
                diagnostics.emit(&inputs);
                return;
            }
        }

        let main = match compiler.build() {
            Ok(object) => object,
            Err(e) => {
                let d = e.into();
                diagnostics.register(d);
                diagnostics.emit(&inputs);
                return;
            }
        };

        if args.dump {
            println!("{main}");
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
            return;
        }
    }
}
