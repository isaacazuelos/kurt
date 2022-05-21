//! A simple garbage collector.
//!
//! The basic design is the same as the garbage collector in [Crafting
//! Interpreters][ci]. It's a super simple mark-sweep collector that keeps all
//! objects in a big linked list. This is basically what Lua does too.
//!
//! The runtime includes reference to the head of a linked list of all allocated
//! objects that's maintained.
//!
//! [ci]: http://craftinginterpreters.com/garbage-collection.html

// TODO: We could be clever and allocate the worklist upfront when we increase
//       the max heap size.

use std::cell::Cell;

use crate::{
    memory::{
        trace::{Trace, WorkList},
        Gc,
    },
    VirtualMachine,
};

pub(crate) struct GCHeader {
    /// The next GC object in our all-objects linked list.
    next: Cell<Option<Gc>>,

    /// Was this object marked as reachable by the last mark phase?
    mark: Cell<bool>,
}

impl Default for GCHeader {
    fn default() -> Self {
        GCHeader {
            next: Cell::new(None),
            mark: Cell::new(false),
        }
    }
}

impl GCHeader {
    /// Is the gc mark bit set?
    pub(crate) fn is_marked(&self) -> bool {
        self.mark.get()
    }

    fn clear_mark(&self) {
        self.mark.set(false);
    }
}

impl VirtualMachine {
    /// Collect garbage, but only if needed.
    #[inline(always)] // inline the fast check, not slow collection.
    pub fn collect_garbage(&mut self) {
        if self.garbage_collection_is_needed() {
            #[cfg(feature = "gc_trace")]
            eprintln!("starting garbage collection");

            self.force_collect_garbage();
        }
    }

    /// Is it time to run a full GC cycle?
    ///
    /// We'll want to add some user-configurable knobs to the runtime for this
    /// eventually.
    pub fn garbage_collection_is_needed(&mut self) -> bool {
        true
    }

    /// Force a full garbage collection cycle, even if it's not needed.
    #[inline(never)] // Collecting is always the slow path
    pub fn force_collect_garbage(&mut self) {
        self.mark();
        self.sweep();
    }

    /// Register a [`Gc`] pointer to be tracked by the runtime.
    ///
    /// # Safety
    ///
    /// The GC must not be tracked by any runtime yet. This should only be
    /// called as part of object creation, after initialization.
    pub(crate) unsafe fn register_gc_ptr(&mut self, ptr: Gc) {
        let header = ptr.deref().gc_header();

        let old_heap_head = self.heap_head;

        debug_assert!(header.next.get().is_none());

        header.next.set(old_heap_head);
        self.heap_head = Some(ptr);
    }

    /// Using [`Runtime`] to access the root set of live objects, we visit every
    /// reachable object and mark it so we can identify the unreachable objects
    /// which must be garbage.
    fn mark(&mut self) {
        #[cfg(feature = "gc_trace")]
        eprintln!("starting mark phase");

        let mut worklist = WorkList::default();

        // This adds the root set.
        self.enqueue_gc_references(&mut worklist);

        // trace
        while let Some(ptr) = worklist.pop() {
            ptr.deref().gc_header().mark.set(true);
            ptr.deref().enqueue_gc_references(&mut worklist);
        }
    }

    /// Deallocate any objects managed by this runtime which are currently
    /// not marked. All objects which remain alive also have their mark cleared.
    fn sweep(&mut self) {
        #[cfg(feature = "gc_trace")]
        eprintln!("starting sweep phase");

        let mut list = self.heap_head.take();

        while let Some(ptr) = list {
            // update list to be the tail.
            let header = ptr.deref().gc_header();
            list = header.next.replace(None);

            if header.is_marked() {
                header.clear_mark();
                unsafe { self.register_gc_ptr(ptr) };
            } else {
                unsafe { self.deallocate(ptr) };
            }
        }
    }
}

impl Trace for VirtualMachine {
    fn enqueue_gc_references(&self, worklist: &mut WorkList) {
        // Values on th stack are reachable.
        for value in self.value_stack().iter() {
            value.enqueue_gc_references(worklist);
        }

        // And all the open captures too
        for capture in self.open_captures() {
            capture.enqueue_gc_references(worklist);
        }
    }
}
