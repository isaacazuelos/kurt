//! Runtime representation of a module.
//!
//! For now it's just a GC wrapper around [`compiler::Module`].

use std::{
    fmt::{self, Debug, Formatter},
    ptr::addr_of_mut,
};

use common::{Get, Index};
use compiler::{Constant, Function, ModuleDebug};

use crate::{
    memory::{Class, ClassId, InitFrom, Object, Trace},
    primitives::PrimitiveOperations,
};

#[derive(PartialEq)]
#[repr(C, align(8))]
pub struct Module {
    base: Object,
    inner: compiler::Module,
}

impl Module {
    pub fn debug_info(&self) -> Option<&ModuleDebug> {
        self.inner.debug_info()
    }
}

impl Get<Function> for Module {
    fn get(&self, index: Index<Function>) -> Option<&Function> {
        self.inner.get(index)
    }
}

impl Get<Constant> for Module {
    fn get(&self, index: Index<Constant>) -> Option<&Constant> {
        self.inner.get(index)
    }
}

impl Debug for Module {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<module>")
    }
}

impl PartialOrd for Module {
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl Trace for Module {
    fn enqueue_gc_references(&self, _worklist: &mut crate::memory::WorkList) {
        // No gc values, yet.
    }
}

impl Class for Module {
    const ID: ClassId = ClassId::Module;
}

impl PrimitiveOperations for Module {
    fn type_name(&self) -> &'static str {
        "module"
    }
}

impl InitFrom<compiler::Module> for Module {
    fn extra_size(_arg: &compiler::Module) -> usize {
        0 // none
    }

    unsafe fn init(ptr: *mut Self, args: compiler::Module) {
        addr_of_mut!((*ptr).inner).write(args)
    }
}
