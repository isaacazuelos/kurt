//! Lists, like pythons for now.

//! Runtime closure representation
use std::{
    fmt::{self, Debug},
    ptr::addr_of_mut,
};

use crate::{
    memory::{
        trace::{Trace, WorkList},
        Class, InitFrom, Object,
    },
    value::Value,
};

#[repr(C, align(8))]
pub struct List {
    base: Object,
    elements: Vec<Value>,
}

impl Class for List {}

impl Trace for List {
    fn enqueue_gc_references(&self, worklist: &mut WorkList) {
        for e in &self.elements {
            e.enqueue_gc_references(worklist);
        }
    }
}

impl Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[ ",)?;
        for e in &self.elements {
            write!(f, "{:?}, ", e)?;
        }
        write!(f, "]",)
    }
}

impl InitFrom<Vec<Value>> for List {
    fn extra_size(_arg: &Vec<Value>) -> usize {
        // This is a fixed-sized.
        0
    }

    unsafe fn init(ptr: *mut Self, arg: Vec<Value>) {
        addr_of_mut!((*ptr).elements).write(arg);
    }
}