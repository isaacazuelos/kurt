//! Run some input as a script.

use std::{fs::File, io::Read, path::PathBuf};

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
        let mut input = String::new();

        if let Err(e) = File::open(&self.filename)
            .and_then(|mut file| file.read_to_string(&mut input))
        {
            eprintln!(
                "Error: cannot read '{}': {}",
                &self.filename.display(),
                e
            );
            std::process::exit(1);
        }

        let mut runtime = Runtime::default();
        runtime.set_tracing(args.trace);

        if let Err(e) = runtime.eval(&input) {
            eprintln!("{}", e);
        }
    }
}
