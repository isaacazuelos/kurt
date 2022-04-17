//! The instructions our VM will use.

use std::fmt::{Display, Formatter, Result};

use crate::{
    constant::Constant, index::Index, local::Local, prototype::Prototype,
};

/// These are the individual instructions that our VM interprets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[rustfmt::skip]
pub enum Op {
    // ## VM control

    /// Stop the program.
    Halt,

    /// Stop the program, but in a way that signals an intent to restart it.
    Yield,

    /// Does nothing.
    Nop,

    // ## Stack Manipulation

    /// Discard the value on the top of the stack, if there is one.
    Pop,

    // ## Loading constant values

    /// Push a `true` to the top of the stack.
    True,

    /// Push a `false` to the top of the stack.
    False,

    /// Push a `()` to the top of the stack.
    Unit,

    /// Load the constant at the specified constant index to the top of the
    /// stack. The currently executing module's constant pool is used.
    LoadConstant(Index<Constant>),

    // ## Loading other kinds of values

    /// Load a local binding.
    LoadLocal(Index<Local>),

    /// Define the top of the stack as a local.
    DefineLocal,

    /// Load a prototype and make a closure from it, placing it on the stack.
    LoadClosure(Index<Prototype>),

    // ## Function Calls

    /// Call a closure on the stack.
    ///
    /// The `u32` is the number of arguments being passed, with the called value
    /// being that far from the top of the stack.
    Call(u32),

    /// Return from the currently executing function.
    Return,

    // ## Branching

    /// Jump to the given opcode index _in the current prototype_
    /// unconditionally.
    Jump(Index<Op>),

    /// Jump to the given index _in the current prototype_ if the top of the
    /// stack if false.
    BranchFalse(Index<Op>),

    // ## Logical Operators
    //
    // We don't have a logical `And` or `Or`, since these would normally be
    // short-circuiting implementations which need branching.

    /// Unary logical negation
    Not,

    // ## Math operators

    /// Unary negation, `-n`
    Neg,

    /// Binary addition
    Add,

    /// Binary subtraction
    Sub,

    /// Binary multiplication
    Mul,

    /// Binary division
    Div,

    /// Binary Exponentiation
    Pow,

    /// Modulus, `n % 2`
    Mod,

    // ## Bitwise Operators

    /// Bitwise And
    BitAnd,

    /// Bitwise Or
    BitOr,

    /// Bitwise Not
    BitNot,

    /// Bitwise XOR
    BitXOR,

    /// Shift Left (logical)
    SLL,

    /// Shift right logical
    SRL,

    /// Shift right arithmetic
    SRA,

    // ## Comparison Operators

    /// Equals
    Eq,

    /// Not Equals
    NEq,

    /// Greater Than
    Gt,

    /// Greater than or equal to
    GEq,

    /// Less Than
    Lt,

    /// Less than or equal to
    LEq,

    // ## Temporary
    //
    // Until the 'real' implementation is done, 

    /// Make a list using the indicated number of arguments on the stack.
    List(u32),

}

impl Display for Op {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            // control
            Op::Halt => write!(f, "Halt"),
            Op::Yield => write!(f, "Yield"),
            Op::Nop => write!(f, "Nop"),
            // stack
            Op::Pop => write!(f, "Pop"),
            // values
            Op::True => write!(f, "True"),
            Op::False => write!(f, "False"),
            Op::Unit => write!(f, "Unit"),
            Op::LoadConstant(i) => write!(f, "LoadConstant {}", i.as_u32()),
            Op::LoadLocal(i) => write!(f, "LoadLocal {}", i.as_u32()),
            Op::DefineLocal => write!(f, "DefineLocal"),
            Op::LoadClosure(i) => write!(f, "LoadClosure {}", i.as_u32()),
            // functions
            Op::Call(i) => write!(f, "Call {}", i),
            Op::Return => write!(f, "Return"),
            Op::Jump(i) => write!(f, "Jump {}", i.as_u32()),
            Op::BranchFalse(i) => write!(f, "BranchFalse {}", i.as_u32()),
            // logic
            Op::Not => write!(f, "Not"),
            // math
            Op::Neg => write!(f, "Neg"),
            Op::Add => write!(f, "Add"),
            Op::Sub => write!(f, "Sub"),
            Op::Mul => write!(f, "Mul"),
            Op::Div => write!(f, "Div"),
            Op::Pow => write!(f, "Pow"),
            Op::Mod => write!(f, "Mod"),
            // bitwise
            Op::BitAnd => write!(f, "BitAnd"),
            Op::BitOr => write!(f, "BitOr"),
            Op::BitNot => write!(f, "BitNot"),
            Op::BitXOR => write!(f, "BitXOR"),
            Op::SLL => write!(f, "SLL"),
            Op::SRL => write!(f, "SRL"),
            Op::SRA => write!(f, "SRA"),
            // comparison
            Op::Eq => write!(f, "Eq"),
            Op::NEq => write!(f, "NEq"),
            Op::Gt => write!(f, "Gt"),
            Op::GEq => write!(f, "GEq"),
            Op::Lt => write!(f, "Lt"),
            Op::LEq => write!(f, "LEq"),
            // temporary
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
