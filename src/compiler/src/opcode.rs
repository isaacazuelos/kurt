//! A listing of opcodes.

use std::fmt::{Display, Formatter, Result};

use common::{i48, u48, Index};

use crate::{Capture, Constant, Function, Local};

type Offset = i32;

/// These are the individual instructions that our VM interprets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[rustfmt::skip]
pub enum Op {
    // ## VM control

    /// Stop the program.
    Halt,

    /// Does nothing.
    Nop,

    // ## Stack Manipulation

    /// Duplicate the value on the top of the stack.
    Dup,
    
    /// Discard the value on the top of the stack, if there is one.
    Pop,

    /// Discard the given number of values from the top of the stack, closing
    /// any open captures in that range, and preserving the value on the top of
    /// the stack.
    Close(u32),

    // ## Loading constant values

    /// Push a `true` to the top of the stack.
    True,

    /// Push a `false` to the top of the stack.
    False,

    /// Push a `()` to the top of the stack.
    Unit,

    // ## Loading Values

    /// An immediate 48-bit signed integer value.
    U48(u48),

    /// An immediate 48-bit signed integer value.
    I48(i48),

    /// Push the currently-executing closure to the top of the stack.
    LoadSelf,

    /// Load the constant at the specified index to the top of the stack. 
    /// 
    /// The currently executing module's constant pool is used.
    LoadConstant(Index<Constant>),

    // ## Loading other kinds of values

    /// Load a local binding.
    LoadLocal(Index<Local>),

    /// Load a non-local binding.
    LoadCapture(Index<Capture>),

    /// Keep the top of the stack as a local.
    DefineLocal,

    /// Make a live function instance from the [`Function`] at the given index,
    /// placing it on the stack.
    LoadFunction(Index<Function>),

    // ## Assignment

    /// Set the value of a local binding.
    SetLocal(Index<Local>),

    /// Set the value of a non-local binding.
    SetCapture(Index<Capture>),

    /// Set a value inside something subscriptable. Expects the new value on
    /// the top of the stack, with the key below it and the target indexable
    /// value below that. Used for `a[b] = c` assignment.
    SetIndex,

    // ## Accessing

    /// Index the item just below the top of the stack by the value on the top
    /// of the stack. Used for `a[b]` style indexing.
    Index,

    // ## Function Calls

    /// Call a closure on the stack.
    ///
    /// The `u32` is the number of arguments being passed, with the called value
    /// being that far from the top of the stack.
    Call(u32),


    /// Return from the currently executing function.
    Return,

    // ## Branching

    /// Jump to the given opcode index _in the currently executing function_
    /// unconditionally.
    Jump(Offset),

    /// Jump to the given index _in the currently executing function_, but only
    /// if the top of the stack is `false`. This pops the stack as well.
    Branch(Offset),

    /// Jump to the given index _in the currently executing function_, but only
    /// if the top of the stack is `false`. This pops the stack as well.
    BranchFalse(Offset),

    // ## Logical Operators
    //
    // We don't have a logical `And` or `Or`, since these would normally be
    // short-circuiting implementations which needs branching.

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

    /// Remainder, `n % 2`
    Rem,

    // ## Bitwise Operators

    /// Bitwise And
    BitAnd,

    /// Bitwise Or
    BitOr,

    /// Bitwise XOR
    BitXOR,

    /// Shift Left
    SHL,

    /// Shift right
    SHR,

    // ## Comparison Operators

    /// Equals
    Eq,

    /// Not Equals
    Ne,

    /// Greater Than
    Gt,

    /// Greater than or equal to
    Ge,

    /// Less Than
    Lt,

    /// Less than or equal to
    Le,

    // ## Temporary
    //
    // Until the 'real' implementation is done, 

    /// Make a list using the indicated number of arguments on the stack.
    List(u32),

    /// Make a tuple
    Tuple(u32, bool),

}

impl Display for Op {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            // We only need to match the ones with embedded arguments
            Op::LoadConstant(i) => write!(f, "LoadConstant {}", i.as_usize()),
            Op::LoadLocal(i) => write!(f, "LoadLocal {}", i.as_usize()),
            Op::LoadFunction(i) => write!(f, "LoadClosure {}", i.as_usize()),
            Op::Call(i) => write!(f, "Call {}", i),
            Op::Jump(i) => write!(f, "Jump {}", i),
            Op::Branch(i) => write!(f, "Branch {}", i),
            Op::BranchFalse(i) => write!(f, "BranchFalse {}", i),
            Op::List(n) => write!(f, "List {n}"),

            // Everything else is the same as what is derived for Debug.
            op => write!(f, "{:?}", op),
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
