//! Run an expression taken from the command line, printing the result.

use compiler::Module;
use diagnostic::{DiagnosticCoordinator, InputCoordinator};
use runtime::Runtime;

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

        let id = inputs.eval_input(self.input.clone());

        let main = match Module::try_from(self.input.as_str()) {
            Ok(o) => o,
            Err(mut d) => {
                d.set_input(Some(id));
                diagnostics.register(d);
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
