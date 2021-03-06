//! Runtime closure representation

use std::{
    cell::RefCell,
    fmt::{self, Debug},
    ops::DerefMut,
    ptr::addr_of_mut,
};

use common::{Get, Index};

use compiler::{Capture, Op};

use crate::{
    classes::{CaptureCell, Module},
    memory::*,
    primitives::PrimitiveOperations,
    Value,
};

use super::Prototype;

#[repr(C, align(8))]
pub struct Function {
    /// The base object required to be a [`Class`].
    base: Object,

    /// The function's prototype, which contains its code and some other metadata.
    prototype: Gc<Prototype>,

    /// The captured values this closure relies on.
    captures: RefCell<Vec<Gc<CaptureCell>>>,
}

impl Function {
    pub fn module(&self) -> Gc<Module> {
        self.prototype.module()
    }

    pub fn prototype(&self) -> Gc<Prototype> {
        self.prototype
    }

    pub fn name(&self) -> Value {
        self.prototype().name()
    }

    pub(crate) fn push_capture_cell(&self, cell: Gc<CaptureCell>) {
        self.captures.borrow_mut().deref_mut().push(cell);
    }

    pub fn get_capture_cell(&self, index: Index<Capture>) -> Gc<CaptureCell> {
        *self
            .captures
            .borrow()
            .get(index.as_usize())
            .expect("capture index out of range")
    }

    pub fn get_op(&self, index: Index<Op>) -> Option<Op> {
        self.prototype().get(index).cloned()
    }

    pub fn capture_count(&self) -> u32 {
        debug_assert!(
            self.captures.borrow().len() <= u32::MAX as usize,
            "LoadCapture takes a u32, so the compiler can't allow more"
        );
        self.captures.borrow().len() as u32
    }
}

impl Class for Function {
    const ID: ClassId = ClassId::Closure;
}

impl PartialOrd for Function {
    /// Closures cannot be ordered.
    ///
    /// What would you even order them by?
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl PartialEq for Function {
    /// Closure equality is identity.
    ///
    /// In theory we could see if they have the same prototype and captures
    /// instead, so multiple closures which we know will behave identically are
    /// equal, but I think that's probably not useful.
    ///
    /// Frankly, I'm not sure I wouldn't rather have this always be false.
    fn eq(&self, other: &Function) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Trace for Function {
    fn enqueue_gc_references(&self, worklist: &mut WorkList) {
        for capture in self.captures.borrow().iter() {
            capture.enqueue_gc_references(worklist);
        }
    }
}

impl PrimitiveOperations for Function {
    fn type_name(&self) -> &'static str {
        "Closure"
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.name() == Value::UNIT {
            write!(f, "<{:?}", self.name())?;
        } else {
            write!(f, "<closure")?;
        }

        if self.capture_count() != 0 {
            write!(f, " [")?;
            for capture in self.captures.borrow().iter() {
                if capture
                    .inline_value()
                    .and_then(|v| Value::as_gc::<Function>(&v))
                    .map(|v| v.identity() == self.identity())
                    .unwrap_or(false)
                {
                    write!(f, "<self>,")?;
                } else {
                    write!(f, "{:?},", capture.contents())?;
                }
            }
            write!(f, "]")?;
        }

        write!(f, ">")
    }
}

impl InitFrom<Gc<Prototype>> for Function {
    fn extra_size(_arg: &Gc<Prototype>) -> usize {
        0
    }

    unsafe fn init(ptr: *mut Self, args: Gc<Prototype>) {
        addr_of_mut!((*ptr).prototype).write(args);
        addr_of_mut!((*ptr).captures).write(RefCell::new(Vec::new()));
    }
}
