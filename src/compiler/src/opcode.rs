//! The instructions our VM will use.

use std::fmt::{Display, Formatter, Result};

use crate::{
    constant::Constant, index::Index, local::Local, prototype::Prototype,
};

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

    /// Load a prototype and make a closure from it, placing it on the stack.
    LoadClosure(Index<Prototype>),

    /// Call a closure on the stack.
    ///
    /// The `u32` is the number of arguments being passed, with the called value
    /// being that far from the top of the stack.
    Call(u32),

    /// Return from the currently executing function.
    Return,

    /// Make a list using the indicated number of arguments on the stack.
    List(u32),
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
            Op::LoadClosure(i) => write!(f, "LoadClosure {}", i.as_u32()),
            Op::Call(i) => write!(f, "Call {}", i),
            Op::Return => write!(f, "Return"),
            Op::List(n) => write!(f, "List {n}"),
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
