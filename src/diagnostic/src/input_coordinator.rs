//! Input Manager

use std::path::PathBuf;

/// A unique ID that corresponds to a piece of input tracked by an
/// [`InputCoordinator`].
///
/// This is used to manage source maps used for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct InputId(usize);

#[derive(Default)]
pub struct InputCoordinator {
    /// Inputs, where [`InputId`] are the corresponding indexes.
    inputs: Vec<Input>,
}

impl InputCoordinator {
    pub fn repl_input(&mut self, buffer: String) -> InputId {
        let id = self.inputs.len();
        self.inputs.push(Input {
            buffer,
            name: Name::Repl,
        });
        InputId(id)
    }

    pub fn eval_input(&mut self, buffer: String) -> InputId {
        let id = self.inputs.len();
        self.inputs.push(Input {
            buffer,
            name: Name::Eval,
        });
        InputId(id)
    }

    pub fn file_input(&mut self, buffer: String, path: PathBuf) -> InputId {
        let id = self.inputs.len();
        self.inputs.push(Input {
            buffer,
            name: Name::File(path),
        });
        InputId(id)
    }

    pub fn get_input_buffer(&self, id: InputId) -> &str {
        self.inputs[id.0].buffer.as_str()
    }

    pub fn get_input_name(&self, id: InputId) -> String {
        match self.inputs[id.0].name() {
            Name::File(path) => (format!("{}", path.display())),
            Name::Repl => (format!("<repl {}>", id.0)),
            Name::Eval if id.0 == 0 => "<eval>".into(),
            Name::Eval => format!("<eval-{}>", id.0),
        }
    }
}

/// A piece of input has a name, and a buffer which contains it's code.
struct Input {
    name: Name,
    buffer: String,
}

impl Input {
    fn name(&self) -> &Name {
        &self.name
    }
}

/// A piece of input is named based on where it came from.
///
/// Usually this is a path to the file we loaded, but it could also be the repl
/// or an an `eval` expression at the command line.
enum Name {
    Repl,
    Eval,
    File(PathBuf),
}
