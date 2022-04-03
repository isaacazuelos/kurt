//! Run an expression taken from the command line, printing the result.

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
        let mut runtime = Runtime::default();

        runtime.set_tracing(args.trace);

        if let Err(e) = runtime.eval(&self.input) {
            eprintln!("{}", e);
        } else {
            println!("{:?}", runtime.last_result());
        }
    }
}
