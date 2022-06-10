//! Lists, like pythons for now.

//! Runtime closure representation
use std::{
    cell::RefCell,
    fmt::{self, Debug},
    ptr::addr_of_mut,
};

use crate::{memory::*, primitives::PrimitiveOperations, value::Value, Error};

#[repr(C, align(8))]
pub struct List {
    base: Object,
    elements: RefCell<Vec<Value>>,
}

impl List {
    /// The number of elements in the list.
    pub fn len(&self) -> usize {
        self.elements.borrow().len()
    }

    /// Is the list the empty list?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn slot(&self, index: Value) -> Result<usize, Error> {
        let i = index
            .as_int()
            .ok_or_else(|| Error::OperationNotSupported {
                type_name: index.type_name(),
                op_name: "indexing by",
            })?
            .as_i64();

        if i >= self.len() as i64 || i < -(self.len() as i64) {
            Err(Error::SubscriptIndexOutOfRange)
        } else if i > 0 {
            Ok(i as usize)
        } else {
            Ok(self.len() - (i.abs() as usize))
        }
    }

    /// Subscript the list by a value.
    pub fn index(&self, index: Value) -> Result<Value, Error> {
        let slot = self.slot(index)?;
        let value = self.elements.borrow()[slot];
        Ok(value)
    }

    pub fn set_index(
        &self,
        index: Value,
        new_value: Value,
    ) -> Result<(), Error> {
        let slot = self.slot(index)?;
        self.elements.borrow_mut()[slot] = new_value;
        Ok(())
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
        for e in self.elements.borrow().iter() {
            e.enqueue_gc_references(worklist);
        }
    }
}

impl Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list()
            .entries(self.elements.borrow().iter())
            .finish()
    }
}

impl InitFrom<Vec<Value>> for List {
    fn extra_size(_arg: &Vec<Value>) -> usize {
        // This is a fixed-sized.
        0
    }

    unsafe fn init(ptr: *mut Self, arg: Vec<Value>) {
        addr_of_mut!((*ptr).elements).write(RefCell::new(arg));
    }
}

impl PrimitiveOperations for List {
    fn type_name(&self) -> &'static str {
        "List"
    }

    fn index(
        &self,
        key: Value,
        _: &mut crate::VirtualMachine,
    ) -> Result<Value, Error> {
        self.index(key)
    }
}
