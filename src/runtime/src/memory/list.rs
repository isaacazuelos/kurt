//! Lists, like pythons for now.

//! Runtime closure representation
use std::{
    fmt::{self, Debug},
    ptr::addr_of_mut,
};

use crate::{
    memory::{
        trace::{Trace, WorkList},
        Class, ClassId, InitFrom, Object,
    },
    primitives::{Error, PrimitiveOperations},
    value::{i48_type::i48, Value},
};

#[repr(C, align(8))]
pub struct List {
    base: Object,
    elements: Vec<Value>,
}

impl List {
    /// The number of elements in the list.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Subscript the list by a value.
    pub fn index(&self, index: Value) -> Result<Value, Error> {
        if let Some(i) = index.as_int() {
            if i < i48::ZERO {
                // i.abs can't overflow, since i came from a Value's 48-bits.
                // the + 1 is safe because we just tested that it's not 0.
                let i = i.as_i64();
                self.index_back((i + 1).abs() as usize)
            } else {
                self.index_front(i.as_i64() as usize)
            }
        } else if let Some(n) = index.as_nat() {
            self.index_front(n.as_u64() as usize)
        } else {
            Err(Error::OperationNotSupported {
                type_name: self.type_name(),
                op_name: "index",
            })
        }
    }

    // Subscript from the back of the list, with 0 being the last element.
    fn index_back(&self, n: usize) -> Result<Value, Error> {
        if n + 1 >= self.len() {
            Err(Error::SubscriptIndexOutOfRange)
        } else {
            let right_index = self.len() - (n + 1);
            self.elements
                .get(right_index)
                .cloned()
                .ok_or(Error::SubscriptIndexOutOfRange)
        }
    }

    fn index_front(&self, n: usize) -> Result<Value, Error> {
        self.elements
            .get(n as usize)
            .cloned()
            .ok_or(Error::SubscriptIndexOutOfRange)
    }
}

impl Class for List {
    const ID: ClassId = ClassId::List;
}

impl PartialEq for List {
    fn eq(&self, other: &Self) -> bool {
        self.elements == other.elements
    }
}

impl PartialOrd for List {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.elements.partial_cmp(&other.elements)
    }
}

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

impl PrimitiveOperations for List {
    fn type_name(&self) -> &'static str {
        "List"
    }

    fn index(
        &self,
        key: Value,
        _: &mut crate::Runtime,
    ) -> Result<Value, Error> {
        self.index(key)
    }
}
