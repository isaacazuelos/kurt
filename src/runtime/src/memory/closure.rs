//! Runtime closure representation
use std::{
    fmt::{self, Debug},
    ptr::addr_of_mut,
};

use compiler::{index::Index, prototype::Prototype};

use crate::{
    memory::{class::Class, trace::Trace, Object},
    module::Module,
};

use super::InitFrom;

#[repr(C, align(8))]
pub struct Closure {
    /// The base object required to be a [`Class`].
    base: Object,
    module: Index<Module>,
    prototype: Index<Prototype>,
}

impl Closure {
    /// The module index for the module this closure was defined in.
    pub fn _module(&self) -> Index<Module> {
        self.module
    }

    /// The prototype index for this closure, in it's original module.
    pub fn prototype(&self) -> Index<Prototype> {
        self.prototype
    }
}

impl Class for Closure {}

impl Trace for Closure {
    fn enqueue_gc_references(&self, _worklist: &mut super::trace::WorkList) {
        // no gc references until we have captures.
    }
}

impl Debug for Closure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<closure {}-{}>",
            self.module.as_usize(),
            self.prototype.as_usize()
        )
    }
}

impl InitFrom<(Index<Module>, Index<Prototype>)> for Closure {
    fn extra_size(_arg: &(Index<Module>, Index<Prototype>)) -> usize {
        // This is a fixed-sized for now, but once we have captures we'll make
        // space for them here.
        0
    }

    unsafe fn init(
        ptr: *mut Self,
        (module, prototype): (Index<Module>, Index<Prototype>),
    ) {
        addr_of_mut!((*ptr).module).write(module);
        addr_of_mut!((*ptr).prototype).write(prototype);
    }
}
