//! The call stack and call frames.

use compiler::Index;

use crate::address::Address;
use crate::stack::Stack;

/// A call frame is information about a specific function call that's either
/// currently running, or could be returned to.
#[derive(Debug, Clone, Copy)]
pub struct CallFrame {
    /// The 'Program Counter' tells us where in some code our VM is currently
    /// executing instructions from.
    pub(crate) pc: Address,

    /// The 'Base Pointer' is the stack index which is the first slot in the
    /// current call frame.
    pub(crate) bp: Index<Stack>,
}

impl Default for CallFrame {
    fn default() -> Self {
        CallFrame {
            pc: Address::new(Index::new(0), Index::new(0), Index::new(0)),
            bp: Index::new(0),
        }
    }
}

impl CallFrame {
    /// Create a new call frame with the given base pointer and program counter.
    pub fn new(pc: Address, bp: Index<Stack>) -> CallFrame {
        CallFrame { pc, bp }
    }
}

/// The call stack is a stack of all [`CallFrame`]s in our virtual machine.
#[derive(Debug, Default)]
pub struct CallStack {
    /// The call frame that's currently executing.
    ///
    /// This is kept inline here to reduce a layer of indirection as the machine
    /// runs.
    current: CallFrame,

    /// The non-active frames are kept here in a stack.
    stack: Vec<CallFrame>,
}

impl CallStack {
    /// The currently executing call frame.
    #[inline]
    pub fn frame(&self) -> CallFrame {
        self.current
    }

    /// A mutable reference to the currently executing call frame.
    #[inline]
    pub fn frame_mut(&mut self) -> &mut CallFrame {
        &mut self.current
    }

    /// Push a new frame onto the stack, which is analogous to jumping into a
    /// function when it's called.
    #[inline]
    pub fn push(&mut self, new_frame: CallFrame) {
        self.stack.push(self.current);
        self.current = new_frame;
    }

    /// Pop the current call frame. Since this leave the previous as not active,
    /// it returns execution to where it previously was.
    ///
    /// Note that this _doesn't_ clean up the value [`Stack`].
    #[inline]
    pub fn pop(&mut self) -> Option<CallFrame> {
        let previous = self.current;
        if let Some(new) = self.stack.pop() {
            self.current = new;
            Some(previous)
        } else {
            None
        }
    }
}
