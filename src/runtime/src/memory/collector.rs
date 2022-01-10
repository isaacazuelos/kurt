//! Garbage Collector trait

use crate::memory::{AnyManaged, Gc, Managed, Tricolour};

pub trait Trace {
    fn trace(&mut self, worklist: &mut WorkList);
}

pub trait Collector {
    type Context;

    /// The main entry point into the garbage collector. This will trigger a
    /// full GC.
    fn collect_garbage(&mut self);
}

pub struct WorkList {
    gray: Vec<Gc<AnyManaged>>,
}

impl WorkList {
    pub fn enqueue<T: Managed>(&mut self, pointer: Gc<T>) {
        let any = Gc::as_any(pointer);

        debug_assert_eq!(Gc::colour(any), Tricolour::Gray);

        self.gray.push(any);
    }

    pub fn pop(&mut self) -> Option<Gc<AnyManaged>> {
        self.gray.pop()
    }
}
