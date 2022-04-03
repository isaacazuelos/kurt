//! The instructions our VM will use.

use std::fmt::{Display, Formatter, Result};

use crate::{constant::Constant, index::Index, local::Local};

/// These are the individual instructions that our VM interprets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// Stop the program for good.
    Halt,

    /// Stop but in a way where it could be resumed.
    Yield,

    /// Does nothing.
    Nop,

    /// Discard the value on the top of the stack, if there is one.
    Pop,

    /// Push a `true` to the top of the stack.
    True,

    /// Push a `false` to the top of the stack.
    False,

    /// Push a `()` to the top of the stack.
    Unit,

    /// Load the constant at the specified constant index to the top of the
    /// stack. The currently executing module's constant pool is used.
    LoadConstant(Index<Constant>),

    /// Load a local binding.
    LoadLocal(Index<Local>),

    /// Define the top of the stack as a local.
    DefineLocal,
}

impl Display for Op {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Op::Halt => write!(f, "Halt"),
            Op::Yield => write!(f, "Yield"),
            Op::Nop => write!(f, "Nop"),
            Op::Pop => write!(f, "Pop"),
            Op::True => write!(f, "True"),
            Op::False => write!(f, "False"),
            Op::Unit => write!(f, "Unit"),
            Op::LoadConstant(i) => write!(f, "LoadConstant {}", i.as_u32()),
            Op::LoadLocal(i) => write!(f, "LoadLocal {}", i.as_u32()),
            Op::DefineLocal => write!(f, "DefineLocal"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_size() {
        // We want our instructions to be 64-bits.
        assert!(std::mem::size_of::<Op>() == std::mem::size_of::<u64>());
        // We want our instructions to be word-sized.
        assert!(std::mem::size_of::<Op>() <= std::mem::size_of::<usize>());
        // Yes, this means we expect to be on a 64-bit machine.
    }
}
