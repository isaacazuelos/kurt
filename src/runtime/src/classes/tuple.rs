//! Tuples!
//!
//! Tuples fixed-length sequences of values which may have a [`Keyword`] tag.

use std::{
    cell::RefCell,
    fmt::{self, Debug},
    ptr::addr_of_mut,
};

use crate::{memory::*, primitives::PrimitiveOperations, value::Value};

use super::Keyword;

#[repr(C, align(8))]
pub struct Tuple {
    base: Object,
    tag: Option<Gc<Keyword>>,
    elements: RefCell<Vec<Value>>,
}

impl Tuple {
    pub fn len(&self) -> usize {
        self.elements.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Class for Tuple {
    const ID: ClassId = ClassId::Tuple;
}

impl Trace for Tuple {
    fn enqueue_gc_references(&self, worklist: &mut WorkList) {
        if let Some(tag) = self.tag {
            worklist.enqueue(tag);
        }

        for element in self.elements.borrow().iter() {
            element.enqueue_gc_references(worklist);
        }
    }
}

impl Debug for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(tag) = self.tag {
            write!(f, ":{}", tag.as_str())?;
        }

        let mut dt = f.debug_tuple("");

        for entry in self.elements.borrow().iter() {
            dt.field(entry);
        }

        dt.finish()
    }
}

impl InitFrom<(Vec<Value>, Option<Gc<Keyword>>)> for Tuple {
    fn extra_size(_arg: &(Vec<Value>, Option<Gc<Keyword>>)) -> usize {
        // This is a fixed-sized.
        0
    }

    unsafe fn init(
        ptr: *mut Self,
        (elements, tag): (Vec<Value>, Option<Gc<Keyword>>),
    ) {
        addr_of_mut!((*ptr).tag).write(tag);
        addr_of_mut!((*ptr).elements).write(RefCell::new(elements));
    }
}

impl PartialEq for Tuple {
    fn eq(&self, other: &Self) -> bool {
        self.elements == other.elements && self.tag == other.tag
    }
}

impl PartialOrd for Tuple {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.tag == other.tag {
            self.elements.partial_cmp(&other.elements)
        } else {
            None
        }
    }
}

impl PrimitiveOperations for Tuple {
    fn type_name(&self) -> &'static str {
        "Tuple"
    }
}
