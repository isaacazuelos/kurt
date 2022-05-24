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
};

use super::Prototype;

#[repr(C, align(8))]
pub struct Closure {
    /// The base object required to be a [`Class`].
    base: Object,

    prototype: Gc<Prototype>,

    // TODO: We should make this inline since we know the max capacity per-closure.
    captures: RefCell<Vec<Gc<CaptureCell>>>,
}

impl Closure {
    pub fn module(&self) -> Gc<Module> {
        self.prototype.module()
    }

    pub fn prototype(&self) -> Gc<Prototype> {
        self.prototype
    }

    pub(crate) fn push_capture_cell(&self, cell: Gc<CaptureCell>) {
        self.captures.borrow_mut().deref_mut().push(cell);
    }

    pub fn get_capture_cell(
        &self,
        index: Index<Capture>,
    ) -> Option<Gc<CaptureCell>> {
        self.captures.borrow().get(index.as_usize()).cloned()
    }

    pub fn get_op(&self, index: Index<Op>) -> Option<Op> {
        self.prototype().get(index).cloned()
    }
}

impl Class for Closure {
    const ID: ClassId = ClassId::Closure;
}

impl PartialOrd for Closure {
    /// Closures cannot be ordered.
    ///
    /// What would you even order them by?
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl PartialEq for Closure {
    /// Closure equality is identity.
    ///
    /// In theory we could see if they have the same prototype and captures
    /// instead, so multiple closures which we know will behave identically are
    /// equal, but I think that's probably not useful.
    ///
    /// Frankly, I'm not sure I wouldn't rather have this always be false.
    fn eq(&self, other: &Closure) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Trace for Closure {
    fn enqueue_gc_references(&self, worklist: &mut WorkList) {
        for capture in self.captures.borrow().iter() {
            capture.enqueue_gc_references(worklist);
        }
    }
}

impl PrimitiveOperations for Closure {
    fn type_name(&self) -> &'static str {
        "Closure"
    }
}

impl Debug for Closure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<closure>",)
    }
}

// impl InitFrom<(Gc<Module>, Index<Function>)> for Closure {
//     fn extra_size((_, _): &(Gc<Module>, Index<Function>)) -> usize {
//         0
//     }

//     unsafe fn init(
//         ptr: *mut Self,
//         (module, function): (Gc<Module>, Index<Function>),
//     ) {
//         addr_of_mut!((*ptr).module).write(module);
//         addr_of_mut!((*ptr).function).write(function);

//         addr_of_mut!((*ptr).captures).write(RefCell::new(Vec::new()));
//     }
// }

impl InitFrom<Gc<Prototype>> for Closure {
    fn extra_size(_arg: &Gc<Prototype>) -> usize {
        0
    }

    unsafe fn init(ptr: *mut Self, args: Gc<Prototype>) {
        addr_of_mut!((*ptr).prototype).write(args);
        addr_of_mut!((*ptr).captures).write(RefCell::new(Vec::new()));
    }
}
