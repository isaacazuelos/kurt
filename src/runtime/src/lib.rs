//! The language runtime interface.

/// A struct that manages an instance of the langauge runtime.
pub struct Runtime {}

impl Default for Runtime {
    fn default() -> Runtime {
        Runtime {}
    }
}

impl Runtime {
    /// Attempts to evaluate some input.
    pub fn eval(&mut self, _input: impl AsRef<[u8]>) {
        todo!("Runtime cannot yet eval code.")
    }
}
