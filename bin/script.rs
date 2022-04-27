//! Run some input as a script.

use std::{fs::File, io::Read, path::PathBuf};

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

        let syntax = match Module::parse(&input) {
            Ok(object) => object,
            Err(e) => return eprintln!("{e}"),
        };

        let main = match compiler::compile(&syntax) {
            Ok(object) => object,
            Err(d) => return eprintln!("{:?}", d),
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
        }
    }
}
