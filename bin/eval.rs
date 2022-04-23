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
        let main = match compiler::compile(&self.input) {
            Ok(object) => object,
            Err(e) => return eprintln!("{e}"),
        };

        if args.dump {
            println!("{main}");
            return;
        }

        let mut runtime = Runtime::new();

        match runtime.load(main) {
            Ok(rt) => rt,
            Err(e) => return eprintln!("{e}"),
        };

        if let Err(e) = runtime.start() {
            eprintln!("{}", e);
        } else {
            println!("{}", runtime.last_result())
        }
    }
}
