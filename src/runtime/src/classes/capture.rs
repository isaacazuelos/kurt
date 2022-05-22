// Upvalues, a GC box that's used to managed captures, and either contains a
// value, or a stack index to a value.

use std::{
    cell::Cell,
    cmp::Ordering,
    fmt::{self, Debug},
    ptr::addr_of_mut,
};

use compiler::Index;

use crate::{
    memory::*, primitives::PrimitiveOperations, value::Value, vm::ValueStack,
    Error,
};

#[derive(Debug, Clone, Copy)]
pub enum CaptureCellContents {
    Inline(Value),
    Stack(Index<ValueStack>),
}

impl CaptureCellContents {
    pub fn get(&self, stack: &ValueStack) -> Result<Value, Error> {
        match self {
            CaptureCellContents::Stack(stack_index) => {
                stack.get(*stack_index).ok_or(Error::StackIndexBelowZero)
            }
            CaptureCellContents::Inline(v) => Ok(*v),
        }
    }
}

#[repr(C, align(8))]
pub struct CaptureCell {
    /// The base object required to be a [`Class`].
    base: Object,
    contents: Cell<CaptureCellContents>,
}

impl CaptureCell {
    pub fn contents(&self) -> CaptureCellContents {
        self.contents.get()
    }

    pub(crate) fn close(&self, value: Value) {
        self.contents.replace(CaptureCellContents::Inline(value));
    }
}

impl Class for CaptureCell {
    const ID: ClassId = ClassId::CaptureCell;
}

impl Trace for CaptureCell {
    fn enqueue_gc_references(&self, worklist: &mut WorkList) {
        match self.contents.get() {
            CaptureCellContents::Inline(v) => {
                v.enqueue_gc_references(worklist);
            }
            CaptureCellContents::Stack(_) => {
                // If the value is on the stack, it's a GC root and we don't
                // need to worry about it.
            }
        }
    }
}

impl PartialEq for CaptureCell {
    fn eq(&self, _other: &Self) -> bool {
        panic!(
            "can't compare upvalues yet, as primitive comparison methods \
            can't look up values still on the stack"
        )
    }
}

impl PartialOrd for CaptureCell {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        None
    }
}

impl PrimitiveOperations for CaptureCell {
    fn type_name(&self) -> &'static str {
        "Upvalue"
    }
}

impl Debug for CaptureCell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.contents.get() {
            CaptureCellContents::Inline(v) => {
                write!(f, "<upvalue inline {:?}>", v)
            }
            CaptureCellContents::Stack(i) => {
                write!(f, "<upvalue stack {:?}>", i)
            }
        }
    }
}

impl InitFrom<Value> for CaptureCell {
    fn extra_size(_: &Value) -> usize {
        0
    }

    unsafe fn init(ptr: *mut Self, value: Value) {
        addr_of_mut!((*ptr).contents)
            .write(Cell::new(CaptureCellContents::Inline(value)));
    }
}

impl InitFrom<Index<ValueStack>> for CaptureCell {
    fn extra_size(_: &Index<ValueStack>) -> usize {
        0
    }

    unsafe fn init(ptr: *mut Self, index: Index<ValueStack>) {
        addr_of_mut!((*ptr).contents)
            .write(Cell::new(CaptureCellContents::Stack(index)));
    }
}
