//! Run an expression taken from the command line, printing the result.

use compiler::Compiler;
use diagnostic::{Diagnostic, DiagnosticCoordinator, InputCoordinator};
use runtime::Runtime;
use syntax::{Module, Parse};

use crate::Args;

/// Evaluate the command line arguments as code and print the result
#[derive(clap::Parser)]
pub struct Evaluate {
    /// The code to evaluate and print
    input: String,
}

impl Evaluate {
    /// Run the subcommand, evaluating and printing it's results.
    pub(crate) fn run(&self, args: &Args) {
        let mut inputs = InputCoordinator::default();
        let mut diagnostics = DiagnosticCoordinator::default();
        let mut compiler = Compiler::default();

        let id = inputs.eval_input(self.input.clone());

        let syntax = match Module::parse(&self.input) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("{e}");
                return;
            }
        };

        match compiler.push(&syntax) {
            Ok(()) => {}
            Err(e) => {
                let mut d = Diagnostic::from(e);
                d.set_input_id(id);
                diagnostics.register(d);
                diagnostics.emit(&inputs);
                return;
            }
        };

        let object = match compiler.build() {
            Ok(o) => o,
            Err(e) => {
                let mut d = Diagnostic::from(e);
                d.set_input_id(id);
                diagnostics.register(d);
                diagnostics.emit(&inputs);
                return;
            }
        };

        if args.dump {
            println!("{object}");
            return;
        }

        let mut runtime = Runtime::new();

        match runtime.load(object) {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("{e}");
                return;
            }
        };

        if let Err(e) = runtime.start() {
            eprintln!("{}", e);
        } else {
            println!("{}", runtime.last_result())
        }
    }
}
