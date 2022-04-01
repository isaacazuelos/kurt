//! Call frames

use compiler::index::Index;

use crate::address::Address;
use crate::stack::Stack;

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
            pc: Address::default(),
            bp: Index::new(0),
        }
    }
}

#[derive(Debug, Default)]
pub struct CallStack {
    current: CallFrame,
    _stack: Vec<CallFrame>,
}

impl CallStack {
    #[inline]
    pub fn frame(&self) -> &CallFrame {
        &self.current
    }

    #[inline]
    pub fn frame_mut(&mut self) -> &mut CallFrame {
        &mut self.current
    }

    #[inline]
    pub fn _push(&mut self, new_frame: CallFrame) {
        self._stack.push(self.current);
        self.current = new_frame;
    }
}
