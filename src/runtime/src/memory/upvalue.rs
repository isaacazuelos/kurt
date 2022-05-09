// Upvalues, a GC box that's used to managed captures, and either contains a
// value, or a stack index to a value.

use std::{
    cmp::Ordering,
    fmt::{self, Debug},
    ptr::addr_of_mut,
};

use compiler::index::Index;

use crate::{
    memory::{
        class::{Class, ClassId},
        trace::Trace,
        InitFrom, Object,
    },
    primitives::PrimitiveOperations,
    stack::Stack,
    value::Value,
};

#[derive(Debug, Clone, Copy)]
pub enum UpvalueContents {
    Inline(Value),
    Stack(Index<Stack>),
}

#[repr(C, align(8))]
pub struct Upvalue {
    /// The base object required to be a [`Class`].
    base: Object,

    contents: UpvalueContents,
}

impl Upvalue {
    pub fn _contents(&self) -> UpvalueContents {
        self.contents
    }
}

impl Class for Upvalue {
    const ID: ClassId = ClassId::Upvalue;
}

impl Trace for Upvalue {
    fn enqueue_gc_references(&self, worklist: &mut super::trace::WorkList) {
        match self.contents {
            UpvalueContents::Inline(v) => v.enqueue_gc_references(worklist),
            UpvalueContents::Stack(_) => {
                // If the value is on the stack, it's a GC root and we don't
                // need to worry about it.
            }
        }
    }
}

impl PartialEq for Upvalue {
    fn eq(&self, _other: &Self) -> bool {
        panic!(
            "can't compare upvalues yet, as primitive comparison methods \
            can't look up values still on the stack"
        )
    }
}

impl PartialOrd for Upvalue {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        None
    }
}

impl PrimitiveOperations for Upvalue {
    fn type_name(&self) -> &'static str {
        "Upvalue"
    }
}

impl Debug for Upvalue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.contents {
            UpvalueContents::Inline(v) => write!(f, "<upvalue inline {:?}>", v),
            UpvalueContents::Stack(i) => write!(f, "<upvalue stack {:?}>", i),
        }
    }
}

impl InitFrom<Value> for Upvalue {
    fn extra_size(_: &Value) -> usize {
        0
    }

    unsafe fn init(ptr: *mut Self, value: Value) {
        addr_of_mut!((*ptr).contents).write(UpvalueContents::Inline(value));
    }
}

impl InitFrom<Index<Stack>> for Upvalue {
    fn extra_size(_: &Index<Stack>) -> usize {
        0
    }

    unsafe fn init(ptr: *mut Self, index: Index<Stack>) {
        addr_of_mut!((*ptr).contents).write(UpvalueContents::Stack(index));
    }
}
