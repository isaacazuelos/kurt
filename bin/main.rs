//! Kurt - A language for fun
use std::{ffi::OsStr, fs::File, io::Read, path::PathBuf};

const INPUT_HELP: &str =
    "The input file is used as the program's main entry point. If no file is \
     provided, stdin is read. If stdin is a tty, the REPL is started.";

const EXPRESSION_HELP: &str =
    "Evaluate EXPRESSION and print the result to standard out.";

fn main() {
    let app = clap::App::new("kurt")
        .version(clap::crate_version!())
        .about("a language for fun")
        .author(clap::crate_authors!())
        .args(&[
            clap::Arg::with_name("input")
                .help(INPUT_HELP)
                .takes_value(true)
                .value_name("FILE")
                .index(1),
            clap::Arg::with_name("expression")
                .help("Evaluate an expression")
                .long_help(EXPRESSION_HELP)
                .short("e")
                .long("expression")
                .alias("eval")
                .value_name("EXPRESSION")
                .takes_value(true)
                .conflicts_with("input"),
        ]);

    let matches = app.get_matches();

    if let Some(filename) = matches.value_of_os("input") {
        run_file(filename);
    }
    if let Some(expression) = matches.value_of_os("expression") {
        run_expression(expression);
    } else {
        todo!("repl not implemented");
    }
}

fn run_file(filename: &OsStr) {
    let path: PathBuf = filename.into();
    let mut buf: Vec<u8> = Vec::new();

    if let Err(e) =
        File::open(&path).and_then(|mut file| file.read_to_end(&mut buf))
    {
        eprintln!("Error: cannot read '{}': {}", path.display(), e);
        std::process::exit(1);
    }

    let mut runtime = runtime::Runtime::default();
    runtime.eval(&buf);
}

fn run_expression(expression: &OsStr) {
    let bytes: Vec<u8>;

    #[cfg(target_family = "windows")]
    {
        bytes = match expression.to_os_string().into_string() {
            Ok(s) => s.as_bytes().into(),
            Err(_) => panic!("expression isn't valid unicode"),
        };
    }

    #[cfg(target_family = "unix")]
    {
        use std::os::unix::prelude::OsStrExt;
        bytes = expression.as_bytes().into();
    }

    let mut runtime = runtime::Runtime::default();

    runtime.eval(&bytes);
}
