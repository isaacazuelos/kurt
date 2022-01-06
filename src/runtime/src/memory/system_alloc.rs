//! A wrapper around Rust's [`std::alloc`], tracking allocations it's
//! responsible for in a big [`BTreeMap`] that maps them to the [`Layout`] of
//! the allocation.

use std::{alloc::Layout, collections::BTreeMap, ptr::NonNull};

use crate::memory::allocator::Allocator;

/// A large object allocator that wraps rust's [`std::alloc::GlobalAlloc`],
/// which is typically the system `malloc`.
///
/// This is basically the dumbest thing that could work right now.
#[derive(Default)]
pub struct SystemAllocator {
    /// The allocations made by this allocator.
    allocations: BTreeMap<NonNull<u8>, Layout>,
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

unsafe impl Allocator for SystemAllocator {
    fn allocate(&mut self, layout: Layout) -> Option<std::ptr::NonNull<u8>> {
        let ptr = NonNull::new(unsafe { std::alloc::alloc_zeroed(layout) });

        if let Some(ptr) = ptr {
            self.allocations.insert(ptr, layout);
        }

        ptr
    }

    unsafe fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.allocations.remove(&ptr);
        std::alloc::dealloc(ptr.as_ptr(), layout)
    }

    fn contains(&self, ptr: NonNull<u8>) -> bool {
        self.allocations.contains_key(&ptr)
    }
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
        unsafe { a.deallocate(ptr1, layout) };
        assert!(!a.contains(ptr1));
    }
}
