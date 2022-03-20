use std::collections::VecDeque;

use super::Gc;

#[derive(Default)]
pub(crate) struct WorkList {
    list: VecDeque<Gc>,
}

impl WorkList {
    /// Add a [`Gc`] value to the work list.
    ///
    /// The value is only actually added it's not marked.
    pub fn enqueue(&mut self, ptr: Gc) {
        if !ptr.deref().gc_header().is_marked() {
            self.list.push_back(ptr);
        }
    }

    /// Remove an object from the worklist (if any are left) to work on it.
    pub fn pop(&mut self) -> Option<Gc> {
        self.list.pop_front()
    }
}

pub(crate) trait Trace {
    /// This is used by the garbage collector to visit every gc pointer retained
    /// by this class.
    ///
    /// Implementations must call [`WorkList::enqueue`] on any references they
    /// keep to other gc objects.
    fn enqueue_gc_references(&self, worklist: &mut WorkList);
}
