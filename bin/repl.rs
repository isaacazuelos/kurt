//! An interactive mode.

use std::io::Write;

use compiler::Compiler;
use runtime::Runtime;
use rustyline::{error::ReadlineError, Editor};

/// Start an interactive session
#[derive(clap::Parser)]
pub struct ReplArgs; // For now there are no repl settings.

impl ReplArgs {
    /// Run a repl with the given settings.
    pub fn run(&self) {
        let repl = Repl::default();
        repl.start()
    }
}

struct Repl {
    editor: Editor<()>,
    runtime: Runtime,
}

impl Default for Repl {
    fn default() -> Self {
        let editor = Editor::<()>::new();
        // TODO: Read history here.
        let runtime = Runtime::default();
        let compiler = Compiler::new();

        Repl { editor, runtime }
    }
}

impl Repl {
    /// The prompt used to ask for more input.
    const PROMPT: &'static str = ">>> ";

    /// Lines which are the result of execution begin with this.
    const RESULT_PROMPT: &'static str = "//> ";

    fn start(mut self) {
        println!(
            "// Please note that the repl does keep previous context yet."
        );
        loop {
            match self.step() {
                Ok(()) => continue,
                Err(ReplError::Clear) => continue,
                Err(ReplError::Exit) => break,
                Err(ReplError::ReadlineError(e)) => {
                    println!("{}", e);
                    println!("  (press control-d to exit)");
                }
            }
        }
    }

    fn step(&mut self) -> Result<(), ReplError> {
        let line = self.read()?;

        self.runtime.eval(&line);
        self.runtime.print(Repl::RESULT_PROMPT);
        self.flush();

        Ok(())
    }

    fn read(&mut self) -> Result<String, ReplError> {
        // TODO: We want to have a continuation prompt when the block of code on
        //       that line could continue.

        let line = self.editor.readline(Repl::PROMPT);
        match line {
            Ok(line) => Ok(line),
            Err(ReadlineError::Interrupted) => {
                // User hit Control-C
                Err(ReplError::Clear)
            }

            Err(ReadlineError::Eof) => {
                // User hit Control-D at end of line, to exit.
                Err(ReplError::Exit)
            }

            Err(e) => Err(ReplError::ReadlineError(e)),
        }
    }

    fn flush(&self) {
        std::io::stdout().flush().expect("failed to flush stdout");
    }
}

#[derive(Debug)]
enum ReplError {
    Clear,
    Exit,
    ReadlineError(ReadlineError),
}

impl std::error::Error for ReplError {}

impl std::fmt::Display for ReplError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplError::Clear => write!(f, "^C"),
            ReplError::Exit => write!(f, "^D"),
            ReplError::ReadlineError(e) => {
                write!(f, "error reading input: {}", e)
            }
        }
    }
}
