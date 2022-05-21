//! Runtime closure representation

use std::{
    cell::RefCell,
    fmt::{self, Debug},
    ops::DerefMut,
    ptr::addr_of_mut,
};

use compiler::{Function, Index, Module};

use crate::{
    memory::{
        class::{Class, ClassId},
        trace::Trace,
        upvalue::Upvalue,
        InitFrom, Object,
    },
    primitives::PrimitiveOperations,
    value::Value,
};

#[repr(C, align(8))]
pub struct Closure {
    /// The base object required to be a [`Class`].
    base: Object,

    module: Index<Module>,
    prototype: Index<Function>,

    // TODO: We should make this inline since we know the max capacity per-closure.
    captures: RefCell<Vec<Value>>,
}

impl Closure {
    /// The module index for the module this closure was defined in.
    pub fn _module(&self) -> Index<Module> {
        self.module
    }

    /// The prototype index for this closure, in it's original module.
    pub fn prototype(&self) -> Index<Function> {
        self.prototype
    }

    pub fn get_capture(&self, i: usize) -> Value {
        self.captures.borrow()[i]
    }

    pub(crate) fn push_capture(&self, upvalue: Value) {
        debug_assert!(
            upvalue
                .as_gc_any()
                .map(|o| o.deref().class_id() == Upvalue::ID)
                == Some(true)
        );

        self.captures.borrow_mut().deref_mut().push(upvalue);
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
    fn enqueue_gc_references(&self, worklist: &mut super::trace::WorkList) {
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
        write!(
            f,
            "<closure {}-{} {:?}>",
            self.module.as_usize(),
            self.prototype.as_usize(),
            self.captures.borrow(),
        )
    }
}

impl InitFrom<(Index<Module>, Index<Function>)> for Closure {
    fn extra_size((_, _): &(Index<Module>, Index<Function>)) -> usize {
        0
    }

    unsafe fn init(
        ptr: *mut Self,
        (module, prototype): (Index<Module>, Index<Function>),
    ) {
        addr_of_mut!((*ptr).module).write(module);
        addr_of_mut!((*ptr).prototype).write(prototype);

        addr_of_mut!((*ptr).captures).write(RefCell::new(Vec::new()));
    }
}
