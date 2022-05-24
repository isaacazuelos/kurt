//! The call stack and call frames.

use common::Index;

use crate::vm::Address;

use super::ValueStack;

/// A call frame is information about a specific function call that's either
/// currently running, or could be returned to.
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// The 'Program Counter' tells us where in some code our VM is currently
    /// executing instructions from.
    pc: Address,

    /// The 'Base Pointer' is the stack index which is the first slot in the
    /// current call frame.
    pub(crate) bp: Index<ValueStack>,
}

impl CallFrame {
    /// Create a new call frame with the given base pointer and program counter.
    pub fn new(pc: Address, bp: Index<ValueStack>) -> CallFrame {
        CallFrame { pc, bp }
    }

    pub fn pc(&self) -> &Address {
        &self.pc
    }

    pub fn pc_mut(&mut self) -> &mut Address {
        &mut self.pc
    }
}

/// The call stack is a stack of all [`CallFrame`]s in our virtual machine.
#[derive(Debug, Default)]
pub struct CallStack {
    /// The call frame that's currently executing.
    ///
    /// This is kept inline here to reduce a layer of indirection as the machine
    /// runs.
    current: Option<CallFrame>,

    /// The non-active frames are kept here in a stack.
    stack: Vec<CallFrame>,
}

impl CallStack {
    /// The number of frames on the call stack.
    pub fn len(&self) -> usize {
        if self.current.is_none() {
            0
        } else {
            self.stack.len() + 1
        }
    }

    /// The currently executing call frame.
    ///
    /// # Panics
    ///
    /// This will panic if the
    #[inline]
    pub fn frame(&self) -> &CallFrame {
        self.current.as_ref().expect("no call frames")
    }

    /// A mutable reference to the currently executing call frame.
    ///
    /// # Panics
    ///
    /// This will panic if there's no current frame.
    #[inline]
    pub fn frame_mut(&mut self) -> &mut CallFrame {
        if let Some(frame) = &mut self.current {
            frame
        } else {
            panic!("no call frames")
        }
    }

    /// Push a new frame onto the stack, which is analogous to jumping into a
    /// function when it's called.
    #[inline]
    pub fn push(&mut self, new_frame: CallFrame) {
        if let Some(frame) = self.current.take() {
            self.stack.push(frame);
        }

        self.current = Some(new_frame);
    }

    /// Pop the current call frame. Since this leave the previous as not active,
    /// it returns execution to where it previously was.
    ///
    /// Note that this _doesn't_ clean up the value [`Stack`].
    #[inline]
    pub fn pop(&mut self) -> Option<CallFrame> {
        let previous = self.current.take()?;
        self.current = self.stack.pop();
        Some(previous)
    }

    /// Iterate over the call stack, from the most recent frame to the oldest.
    pub fn iter(&self) -> CallStackIterator<'_> {
        CallStackIterator {
            inner: self,
            index: 0,
        }
    }

    pub fn get(&self, index: usize) -> Option<&CallFrame> {
        if index == 0 {
            self.current.as_ref()
        } else {
            self.stack.iter().nth_back(index - 1)
        }
    }
}

/// Iterator over the call stack's frames, this starts at the current frame and
/// works backwards.
pub struct CallStackIterator<'a> {
    inner: &'a CallStack,
    index: usize,
}

impl<'a> Iterator for CallStackIterator<'a> {
    type Item = &'a CallFrame;

    fn next(&mut self) -> Option<Self::Item> {
        let frame = self.inner.get(self.index)?;
        self.index = self.index.checked_add(1)?;
        Some(frame)
    }
}

impl<'a> DoubleEndedIterator for CallStackIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let frame = self.inner.get(self.index)?;
        self.index = self.index.checked_sub(1)?;
        Some(frame)
    }
}

impl<'a> ExactSizeIterator for CallStackIterator<'a> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}
