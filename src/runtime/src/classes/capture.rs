// Upvalues, a GC box that's used to managed captures, and either contains a
// value, or a stack index to a value.

use std::{
    cell::Cell,
    cmp::Ordering,
    fmt::{self, Debug},
    ptr::addr_of_mut,
};

use common::Index;

use crate::{
    memory::*, primitives::PrimitiveOperations, value::Value, vm::Stack,
};

#[derive(Clone, Copy)]
pub enum CaptureCellContents {
    /// The value of the cell is kept inside the cell.
    ///
    /// This is what lua would call a 'closed upvalue'.
    Inline(Value),

    /// The value of the cell is on the stack at the given index.
    ///
    /// This is what lua would call an 'open upvalue'.
    Stack(Index<Stack>),
}

impl CaptureCellContents {
    pub fn get(&self, stack: &Stack) -> Value {
        match self {
            CaptureCellContents::Stack(stack_index) => stack[*stack_index],
            CaptureCellContents::Inline(v) => *v,
        }
    }
}

impl Debug for CaptureCellContents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CaptureCellContents::Inline(v) => write!(f, "closed {:?}", v),
            CaptureCellContents::Stack(s) => {
                write!(f, "open {:?}", s)
            }
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
    pub fn is_open(&self) -> bool {
        match self.contents.get() {
            CaptureCellContents::Inline(_) => true,
            CaptureCellContents::Stack(_) => false,
        }
    }

    pub fn stack_index(&self) -> Option<Index<Stack>> {
        match self.contents.get() {
            CaptureCellContents::Inline(_) => None,
            CaptureCellContents::Stack(i) => Some(i),
        }
    }

    pub fn inline_value(&self) -> Option<Value> {
        match self.contents.get() {
            CaptureCellContents::Inline(v) => Some(v),
            CaptureCellContents::Stack(_) => None,
        }
    }

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
        write!(f, "<capture {:?}>", self.contents.get())
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

impl InitFrom<Index<Stack>> for CaptureCell {
    fn extra_size(_: &Index<Stack>) -> usize {
        0
    }

    unsafe fn init(ptr: *mut Self, index: Index<Stack>) {
        addr_of_mut!((*ptr).contents)
            .write(Cell::new(CaptureCellContents::Stack(index)));
    }
}
