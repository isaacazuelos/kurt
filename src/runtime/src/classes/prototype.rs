//! Runtime function prototype representation.
//!
//! By function prototype, I mean this is a description of a function, but not
//! the actual callable instance. This is the object that hold a function's code
//! and metadata, but to actually call it you need to create an instance of the
//! function which can be responsible for it's captured values. That instance is
//! a [`Function`][crate::classes::Function].

use std::{
    fmt::{self, Debug, Formatter},
    ptr::addr_of_mut,
};

use common::{Get, Index};
use compiler::{Capture, FunctionDebug, Op};

use crate::{classes::Module, memory::*, primitives::PrimitiveOperations};

#[derive(PartialEq)]
#[repr(C, align(8))]
pub struct Prototype {
    /// The base object required to be a [`Class`].
    base: Object,

    module: Gc<Module>,
    inner: compiler::Function,
}

impl Prototype {
    pub fn module(&self) -> Gc<Module> {
        self.module
    }

    pub fn debug_info(&self) -> Option<&FunctionDebug> {
        self.inner.debug_info()
    }

    pub(crate) fn capture_count(&self) -> u32 {
        self.inner.capture_count()
    }

    pub(crate) fn parameter_count(&self) -> u32 {
        self.inner.parameter_count()
    }
}

impl Get<Op> for Prototype {
    fn get(&self, index: Index<Op>) -> Option<&Op> {
        self.inner.get(index)
    }
}

impl Get<Capture> for Prototype {
    fn get(&self, index: Index<Capture>) -> Option<&Capture> {
        self.inner.get(index)
    }
}

impl PartialOrd for Prototype {
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl Class for Prototype {
    const ID: ClassId = ClassId::Prototype;
}

impl Debug for Prototype {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<prototype>")
    }
}

impl Trace for Prototype {
    fn enqueue_gc_references(&self, worklist: &mut WorkList) {
        worklist.enqueue(self.module)
    }
}

impl PrimitiveOperations for Prototype {
    fn type_name(&self) -> &'static str {
        "prototype"
    }
}

impl InitFrom<(Gc<Module>, compiler::Function)> for Prototype {
    fn extra_size(_arg: &(Gc<Module>, compiler::Function)) -> usize {
        0 // none
    }

    unsafe fn init(
        ptr: *mut Self,
        (module, inner): (Gc<Module>, compiler::Function),
    ) {
        addr_of_mut!((*ptr).inner).write(inner);
        addr_of_mut!((*ptr).module).write(module);
    }
}
