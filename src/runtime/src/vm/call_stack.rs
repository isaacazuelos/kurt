//! The call stack and call frames.

use common::Index;

use crate::{classes::Module, memory::Gc, vm::Address};

use super::ValueStack;

/// A call frame is information about a specific function call that's either
/// currently running, or could be returned to.
#[derive(Debug, Clone, Copy)]
pub struct CallFrame {
    /// The 'Program Counter' tells us where in some code our VM is currently
    /// executing instructions from.
    pub(crate) pc: Address,

    /// The 'Base Pointer' is the stack index which is the first slot in the
    /// current call frame.
    pub(crate) bp: Index<ValueStack>,
}

impl CallFrame {
    /// Create a new call frame with the given base pointer and program counter.
    pub fn new(pc: Address, bp: Index<ValueStack>) -> CallFrame {
        CallFrame { pc, bp }
    }
}

/// The call stack is a stack of all [`CallFrame`]s in our virtual machine.
#[derive(Debug)]
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
    /// # Safety
    ///
    /// We have a problem where the call stack can't be empty and we've inlined
    /// the current frame for performance, but we also can't produce the
    /// Gc<Module> needed for the initial address until after the VM is
    /// initialized. So we crate a fake dangling CallStack.
    pub(crate) unsafe fn new_dangling() -> CallStack {
        CallStack {
            current: CallFrame {
                pc: Address {
                    module: Gc::dangling(),
                    function: Index::START,
                    instruction: Index::START,
                },
                bp: Index::START,
            },
            stack: Vec::new(),
        }
    }

    pub fn new(main: Gc<Module>) -> CallStack {
        CallStack {
            current: CallFrame {
                pc: Address {
                    module: main,
                    function: Index::START,
                    instruction: Index::START,
                },
                bp: Index::START,
            },
            stack: Vec::new(),
        }
    }

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

    pub fn iter(&self) -> CallStackIterator<'_> {
        CallStackIterator {
            inner: self,
            index: Some(self.stack.len()),
        }
    }
}

/// Iterator over the call stack's frames, this starts at the current frame and
/// works backwards.
pub struct CallStackIterator<'a> {
    /// The call stack we're iterating over.
    inner: &'a CallStack,

    /// `None` means we already went over index 0.
    ///
    /// If it's at inner.stack.len() then we're at the current frame.
    ///
    /// If it's larger then we next_backed over the current frame.
    index: Option<usize>,
}

impl<'a> Iterator for CallStackIterator<'a> {
    type Item = CallFrame;

    fn next(&mut self) -> Option<Self::Item> {
        match self.index {
            // already exhausted the stack
            None => None,

            // if it's at the current or past the current frame, that's the next one.
            Some(n) if n >= self.inner.stack.len() => {
                self.index = self.inner.stack.len().checked_sub(1);
                Some(self.inner.current)
            }

            // otherwise, we decrement the index and get the frame it pointed to
            Some(n) => {
                self.index = n.checked_sub(1);
                self.inner.stack.get(n).cloned()
            }
        }
    }
}

impl<'a> ExactSizeIterator for CallStackIterator<'a> {
    fn len(&self) -> usize {
        self.inner.stack.len() + 1
    }
}
