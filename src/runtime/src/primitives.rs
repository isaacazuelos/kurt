//! A trait that defines how different operations work.
//!
//! The idea is that each type we expose to the runtime implements this trait,
//! and overrides the specific operations it supports. This lets us dispatch
//! operations like `Add` or `Subscript` at runtime more easily.

// Other things we might want:
//
// - Hash
// - Display/Debug
// - Call
// - Parse

use std::cmp::Ordering;

use crate::{Runtime, Value};

#[derive(Debug)]
pub enum Error {
    OperationNotSupported {
        type_name: &'static str,
        op_name: &'static str,
    },

    SubscriptIndexOutOfRange,

    CastError {
        from: &'static str,
        to: &'static str,
    },
}

pub trait PrimitiveOperations: Sized {
    fn type_name(&self) -> &'static str;

    fn neg(&self, _: &mut Runtime) -> Result<Value, Error> {
        Err(Error::OperationNotSupported {
            type_name: self.type_name(),
            op_name: "neg",
        })
    }
    fn not(&self, _: &mut Runtime) -> Result<Value, Error> {
        Err(Error::OperationNotSupported {
            type_name: self.type_name(),
            op_name: "not",
        })
    }

    fn add(&self, _: Value, _: &mut Runtime) -> Result<Value, Error> {
        Err(Error::OperationNotSupported {
            type_name: self.type_name(),
            op_name: "add",
        })
    }

    fn sub(&self, _: Value, _: &mut Runtime) -> Result<Value, Error> {
        Err(Error::OperationNotSupported {
            type_name: self.type_name(),
            op_name: "sub",
        })
    }

    fn mul(&self, _: Value, _: &mut Runtime) -> Result<Value, Error> {
        Err(Error::OperationNotSupported {
            type_name: self.type_name(),
            op_name: "mul",
        })
    }

    fn div(&self, _: Value, _: &mut Runtime) -> Result<Value, Error> {
        Err(Error::OperationNotSupported {
            type_name: self.type_name(),
            op_name: "div",
        })
    }

    fn pow(&self, _: Value, _: &mut Runtime) -> Result<Value, Error> {
        Err(Error::OperationNotSupported {
            type_name: self.type_name(),
            op_name: "pow",
        })
    }

    fn rem(&self, _: Value, _: &mut Runtime) -> Result<Value, Error> {
        Err(Error::OperationNotSupported {
            type_name: self.type_name(),
            op_name: "rem",
        })
    }

    fn bitand(&self, _: Value, _: &mut Runtime) -> Result<Value, Error> {
        Err(Error::OperationNotSupported {
            type_name: self.type_name(),
            op_name: "bitand",
        })
    }

    fn bitor(&self, _: Value, _: &mut Runtime) -> Result<Value, Error> {
        Err(Error::OperationNotSupported {
            type_name: self.type_name(),
            op_name: "bitor",
        })
    }

    fn bitxor(&self, _: Value, _: &mut Runtime) -> Result<Value, Error> {
        Err(Error::OperationNotSupported {
            type_name: self.type_name(),
            op_name: "bitxor",
        })
    }

    fn shl(&self, _: Value, _: &mut Runtime) -> Result<Value, Error> {
        Err(Error::OperationNotSupported {
            type_name: self.type_name(),
            op_name: "shl",
        })
    }

    fn shr(&self, _: Value, _: &mut Runtime) -> Result<Value, Error> {
        Err(Error::OperationNotSupported {
            type_name: self.type_name(),
            op_name: "shr",
        })
    }

    fn is_truthy(&self) -> bool {
        true
    }

    fn index(&self, _: Value, _: &mut Runtime) -> Result<Value, Error> {
        Err(Error::OperationNotSupported {
            type_name: self.type_name(),
            op_name: "index",
        })
    }

    fn cmp(&self, _: &Self) -> Option<Ordering> {
        None
    }

    fn eq(&self, other: &Self) -> Option<bool> {
        PrimitiveOperations::cmp(self, other).map(Ordering::is_eq)
    }

    fn ne(&self, other: &Self) -> Option<bool> {
        PrimitiveOperations::cmp(self, other).map(Ordering::is_ne)
    }

    fn ge(&self, other: &Self) -> Option<bool> {
        PrimitiveOperations::cmp(self, other).map(Ordering::is_ne)
    }

    fn gt(&self, other: &Self) -> Option<bool> {
        PrimitiveOperations::cmp(self, other).map(Ordering::is_gt)
    }

    fn le(&self, other: &Self) -> Option<bool> {
        PrimitiveOperations::cmp(self, other).map(Ordering::is_le)
    }

    fn lt(&self, other: &Self) -> Option<bool> {
        PrimitiveOperations::cmp(self, other).map(Ordering::is_lt)
    }
}
