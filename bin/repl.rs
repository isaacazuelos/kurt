//! An interactive mode.

use std::io::Write;

use compiler::ModuleBuilder;
use diagnostic::{
    Diagnostic, DiagnosticCoordinator, InputCoordinator, InputId,
};
use runtime::Runtime;
use rustyline::{error::ReadlineError, Editor};
use syntax::{Module, Parse};

use crate::Args;
/// Start an interactive session
#[derive(clap::Parser)]
pub struct Repl; // For now there are no repl settings.

impl Repl {
    /// Run a repl with the given settings.
    pub(crate) fn run(&self, args: &Args) {
        let repl = ReplState::new(args);
        repl.start()
    }
}

struct ReplState {
    dump: bool,
    editor: Editor<()>,
    runtime: Runtime,
    module: ModuleBuilder,
    diagnostics: DiagnosticCoordinator,
    inputs: InputCoordinator,
}

impl ReplState {
    /// The prompt used to ask for more input.
    const PROMPT: &'static str = ">>> ";

    /// Lines which are the result of execution begin with this.
    const RESULT_PROMPT: &'static str = "//> ";

    fn new(args: &Args) -> ReplState {
        let editor = Editor::<()>::new();

        // TODO: Read history here.

        ReplState {
            dump: args.dump,
            editor,
            runtime: Runtime::default(),
            module: ModuleBuilder::default(),
            diagnostics: DiagnosticCoordinator::default(),
            inputs: InputCoordinator::default(),
        }
    }

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
                Err(ReplError::Step) => {
                    self.diagnostics.emit(&self.inputs);
                    self.diagnostics.clear();
                }
            }
        }
    }

    fn step(&mut self) -> Result<(), ReplError> {
        let id = self.read()?;

        let syntax = match Module::parse(self.inputs.get_input_buffer(id)) {
            Ok(syntax) => syntax,
            Err(error) => {
                let mut diagnostic = Diagnostic::from(error);
                diagnostic.set_input(Some(id));
                self.diagnostics.register(diagnostic);
                return Err(ReplError::Step);
            }
        };

        if let Err(error) = self.module.push_syntax(&syntax) {
            let mut diagnostic = Diagnostic::from(error);
            diagnostic.set_input(Some(id));
            self.diagnostics.register(diagnostic);
            return Err(ReplError::Step);
        }

        let new_main = self.module.build();

        if self.dump {
            println!("{}", new_main);
        }

        if let Err(error) = self.runtime.reload_main(new_main) {
            let mut diagnostic = Diagnostic::new(format!("{error}"));
            diagnostic.set_input(Some(id));
            self.diagnostics.register(diagnostic);
            return Err(ReplError::Step);
        }

        if let Err(error) = self.runtime.resume() {
            let mut diagnostic = Diagnostic::new(format!("{error}"));
            diagnostic.set_input(Some(id));
            self.diagnostics.register(diagnostic);
            return Err(ReplError::Step);
        }

        println!(
            "{} {}",
            ReplState::RESULT_PROMPT,
            self.runtime.last_result()
        );

        self.flush();

        Ok(())
    }

    fn read(&mut self) -> Result<InputId, ReplError> {
        // TODO: We want to have a continuation prompt when the block of code on
        //       that line could continue.

        let line = self.editor.readline(ReplState::PROMPT);
        match line {
            Ok(line) => Ok(self.inputs.repl_input(line)),

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
    Step,

    Readline(ReadlineError),
}
