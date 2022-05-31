//! Exit conditions for the virtual machine.

/// Each [`Exit`] is a reason a [`VirtualMachine`][crate::VirtualMachine] may have stopped running (which
/// isn't an [`Error`][crate::Error]).
#[derive(Debug, PartialEq)]
pub enum Exit {
    /// The runtime hit the end of it's code.
    Halt,

    /// The runtime hit a yield point, which for now means the end of repl code.
    Yield,
}
