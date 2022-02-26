//! An interactive mode.

use std::io::Write;

use compiler::Compiler;
use runtime::Runtime;
use rustyline::{error::ReadlineError, Editor};
use syntax::{Module, Parse};

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
    compiler: Compiler,
}

impl Default for Repl {
    fn default() -> Self {
        let editor = Editor::<()>::new();
        // TODO: Read history here.
        let runtime = Runtime::default();
        let compiler = Compiler::new();

        Repl {
            editor,
            runtime,
            compiler,
        }
    }
}

impl Repl {
    /// The prompt used to ask for more input.
    const PROMPT: &'static str = ">>> ";

    /// Lines which are the result of execution begin with this.
    const RESULT_PROMPT: &'static str = "//> ";

    fn start(mut self) {
        loop {
            match self.step() {
                Ok(()) => continue,
                Err(ReplError::Clear) => continue,
                Err(ReplError::Exit) => break,
                Err(ReplError::Readline(e)) => {
                    println!("{}", e);
                    println!("  (press control-d to exit)");
                }
                Err(other) => {
                    println!("{}", other);
                }
            }
        }
    }

    fn step(&mut self) -> Result<(), ReplError> {
        let input = self.read()?;

        let syntax = Module::parse(&input)?;

        // We clone and compile, so that if there's a compile time error we
        // still have `self.compiler` as the last-known-good state.
        let compiler = self.compiler.clone();
        self.compiler = compiler.compile(&syntax)?;

        let updated_main = self.compiler.build();
        self.runtime.reload_main(updated_main)?;

        self.runtime.resume()?;

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

            Err(e) => Err(ReplError::Readline(e)),
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

    Readline(ReadlineError),
    Syntax(syntax::Error),
    CompileTime(compiler::Error),
    Runtime(runtime::Error),
}

impl std::error::Error for ReplError {}

impl std::fmt::Display for ReplError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplError::Clear => write!(f, "^C"),
            ReplError::Exit => write!(f, "^D"),
            ReplError::Readline(e) => write!(f, "{}", e),
            ReplError::Syntax(e) => write!(f, "{}", e),
            ReplError::CompileTime(e) => write!(f, "{}", e),
            ReplError::Runtime(e) => write!(f, "{}", e),
        }
    }
}

impl From<syntax::Error> for ReplError {
    fn from(e: syntax::Error) -> Self {
        ReplError::Syntax(e)
    }
}

impl From<compiler::Error> for ReplError {
    fn from(e: compiler::Error) -> Self {
        ReplError::CompileTime(e)
    }
}

impl From<runtime::Error> for ReplError {
    fn from(e: runtime::Error) -> Self {
        ReplError::Runtime(e)
    }
}
