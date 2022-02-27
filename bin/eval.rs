//! Run an expression taken from the command line, printing the result.

use runtime::Runtime;

/// Evaluate the command line arguments as code and print the result
#[derive(clap::Parser)]
pub struct Evaluate {
    /// The code to evaluate and print
    input: String,
}

impl Evaluate {
    /// Run the subcommand, evaluating and printing it's results.
    pub fn run(&self) {
        let mut runtime = Runtime::default();

        if let Err(e) = runtime.eval(&self.input) {
            eprintln!("{}", e);
            println!("runtime state: {:#?}", runtime);
        } else {
            println!("{:?}", runtime.last_result());
        }
    }
}
