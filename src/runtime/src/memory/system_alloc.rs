//! A wrapper around Rust's [`std::alloc`], tracking allocations it's
//! responsible for in a big [`BTreeMap`] that maps them to the [`Layout`] of
//! the allocation.

use std::{alloc::Layout, collections::BTreeMap, ptr::NonNull};

use crate::memory::{
    allocator::Allocator,
    collector::{Collector, Trace},
    AnyManaged, Gc, Tricolour,
};

use super::collector::WorkList;

/// A large object allocator that wraps rust's [`std::alloc::GlobalAlloc`],
/// which is typically the system `malloc`.
///
/// This is basically the dumbest thing that could work right now.
pub struct SystemAllocator {
    /// The allocations made by this allocator.
    allocations: BTreeMap<NonNull<u8>, Layout>,
    /// The sum of sizes of all allocations.
    bytes_allocated: usize,
    /// The current 'capacity' of the heap, exceeding this will trigger a
    /// collection.
    next_gc: usize,
}

impl Drop for SystemAllocator {
    fn drop(&mut self) {
        // We need to manually implement drop to deallocate the pointers in the
        // allocations.
        for (ptr, layout) in self.allocations.iter() {
            unsafe { std::alloc::dealloc(ptr.as_ptr(), *layout) };
        }
    }
}

impl Default for SystemAllocator {
    fn default() -> Self {
        SystemAllocator {
            allocations: BTreeMap::new(),
            bytes_allocated: 0,
            next_gc: SystemAllocator::DEFAULT_HEAP_SIZE,
        }
    }
}

unsafe impl Allocator for SystemAllocator {
    fn allocate(&mut self, layout: Layout) -> Option<std::ptr::NonNull<u8>> {
        if self.bytes_allocated + layout.size() >= self.next_gc {
            // self.collect_garbage(); // how do we get the root set here?
        }

        let ptr = NonNull::new(unsafe { std::alloc::alloc_zeroed(layout) });

        if let Some(ptr) = ptr {
            self.allocations.insert(ptr, layout);
            self.bytes_allocated += layout.size();
        }

        ptr
    }

    unsafe fn deallocate(&mut self, ptr: NonNull<u8>) {
        if let Some(layout) = self.allocations.remove(&ptr) {
            self.bytes_allocated -= layout.size();
            std::alloc::dealloc(ptr.as_ptr(), layout);
        }
    }

    fn contains(&self, ptr: NonNull<u8>) -> bool {
        self.allocations.contains_key(&ptr)
    }
}

impl Collector for SystemAllocator {
    type Context = Vec<Gc<AnyManaged>>;

    fn collect_garbage(&mut self) {
        let mut worklist = self.mark_roots();
        self.trace_references(&mut worklist);
        self.sweep();
    }
}

// Methods for supporting collection
impl SystemAllocator {
    const DEFAULT_HEAP_SIZE: usize = 1024; // 1 KB

    /// Mark the objects in the root set, producing the initial worklist.
    fn mark_roots(&mut self) -> WorkList {
        todo!()
    }

    /// Using the provided work list, trace all the references marking
    /// everything live. Basically the 'mark' part of our 'mark-sweep'.
    fn trace_references(&mut self, worklist: &mut WorkList) {
        while let Some(mut object) = worklist.pop() {
            unsafe { Gc::set_colour(object, Tricolour::Black) };
            object.trace(worklist);
        }
    }

    /// Deallocate everything that's not marked.
    fn sweep(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_contains() {
        let mut a = SystemAllocator::default();
        let layout = Layout::from_size_align(128, 16).unwrap();
        let ptr1 = a.allocate(layout).unwrap();

        assert!(a.contains(ptr1));
        unsafe { a.deallocate(ptr1) };
        assert!(!a.contains(ptr1));
    }
}
