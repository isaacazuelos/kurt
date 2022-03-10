//! GC Pointers

use std::{fmt, ptr::NonNull};

use super::string::String;
use super::Object;

#[derive(Debug)]
#[repr(transparent)]
pub struct GcObj {
    ptr: NonNull<Object>,
}

impl GcObj {
    pub(crate) unsafe fn from_non_null(ptr: NonNull<Object>) -> GcObj {
        GcObj { ptr }
    }

    pub(crate) fn deref(&self) -> &Object {
        unsafe { self.ptr.as_ref() }
    }
}

impl Clone for GcObj {
    fn clone(&self) -> Self {
        GcObj { ptr: self.ptr }
    }
}

impl Copy for GcObj {}

impl PartialEq for GcObj {
    fn eq(&self, other: &Self) -> bool {
        let lhs = self.deref();
        let rhs = other.deref();

        lhs == rhs
    }
}

impl fmt::Display for GcObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let obj = self.deref();
        match obj.tag() {
            super::object::Tag::String => {
                let s = obj.downcast::<String>().unwrap();
                write!(f, "{}", s.as_str())
            }
        }
    }
}
