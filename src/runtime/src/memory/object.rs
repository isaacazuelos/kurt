//! Our common base object type that all run time values which allocate are
//! based on.

use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
};

use crate::{
    memory::{
        class::{Class, ClassId},
        closure::Closure,
        collector::GCHeader,
        keyword::Keyword,
        list::List,
        string::String,
        trace::{Trace, WorkList},
    },
    primitives::{Error, PrimitiveOperations},
    value::Value,
    Runtime,
};

macro_rules! dispatch {
    ($f: path, $obj: ident, $( $arg: expr, )*) => {
        match $obj.class_id {
            ClassId::Closure => $f( $obj.downcast::<Closure>().unwrap(), $( $arg, )*),
            ClassId::Keyword => $f( $obj.downcast::<Keyword>().unwrap(), $( $arg, )*),
            ClassId::List    => $f( $obj.downcast::<List>().unwrap(), $( $arg, )*),
            ClassId::String  => $f( $obj.downcast::<String>().unwrap(), $( $arg, )*),
        }
    };
}

/// All our runtime values which live on the heap must share some common
/// metadata and methods to allow the runtime to be aware of them and manage
/// their resources. This is done by placing this common metadata first in any
/// of our types.
///
/// The different concrete types of values all implement [`Class`].
///
/// # Notes
///
/// There's deliberately no way to create an [`Object`] that's not some other
/// concrete [`Class`] (using the [`Runtime::make`][Runtime::make])
#[repr(C, align(8))]
pub(crate) struct Object {
    /// The size (in bytes) of the allocation belonging to this [`Object`].
    size: usize,

    /// Data tracked by the garbage collector.
    gc_header: GCHeader,

    /// The concrete type of the object, it's [`Class`]. This is used to recover
    /// the type of an [`Object`] and safely downcast it.
    class_id: ClassId,
}

impl Object {
    /// The alignment used for all objects.
    pub const ALIGN: usize = 8; // Must keep in sync with repr directive.

    /// The specific [`Class`] of this object.
    pub(crate) fn class_id(&self) -> ClassId {
        self.class_id
    }

    /// Attempt to cast the object as an reference to a specific [`Class`].
    ///
    /// This return's `None` if the object is not the right class.
    pub fn downcast<C: Class>(&self) -> Option<&C> {
        if self.class_id() == C::ID {
            Some(unsafe { std::mem::transmute::<_, _>(self) })
        } else {
            None
        }
    }

    pub fn enqueue_gc_references(&self, worklist: &mut WorkList) {
        dispatch!(Trace::enqueue_gc_references, self, worklist,)
    }

    /// A reference to this object's GC state.
    ///
    /// # Note
    ///
    /// There's no `gc_header_mut` method, instead we use interior mutability
    /// and the required access to those methods is kept private inside the
    /// collector.
    pub fn gc_header(&self) -> &GCHeader {
        &self.gc_header
    }

    /// Initialize the common object fields for some object.
    ///
    /// # Safety
    ///
    /// The raw pointer `ptr` must be non-null and uninitialized, it must point
    /// to an allocation that's `size` bytes long.
    ///
    /// This should be called on an object _before_ the specific class init
    /// methods are called.
    ///
    /// Since the concrete type will be set to the type parameter `C`, we need
    /// to know that the object is intended for use as that class.
    pub(crate) unsafe fn init<C: Class>(ptr: *mut Object, size: usize) {
        ptr.write(Object {
            size,
            gc_header: GCHeader::default(),
            class_id: C::ID,
        })
    }

    /// The size of the object's underlying allocation, in bytes.
    pub(crate) fn size(&self) -> usize {
        self.size
    }
}

impl Value {
    /// Use a value as an instance of some [`Class`] `C`, if it is one.
    pub(crate) fn use_as<C, F, R>(&self, inner: F) -> Result<R, Error>
    where
        C: Class,
        F: FnOnce(&C) -> Result<R, Error>,
    {
        if let Some(object) = self.as_object() {
            if let Some(instance) = object.deref().downcast() {
                return inner(instance);
            }
        }

        Err(Error::CastError {
            from: self.type_name(),
            to: C::ID.name(),
        })
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        if other.class_id() == self.class_id() {
            dispatch!(PartialEq::eq, self, other.downcast().unwrap(),)
        } else {
            // Different types are always unequal.
            false
        }
    }
}

impl PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if other.class_id() == self.class_id() {
            dispatch!(PartialOrd::partial_cmp, self, other.downcast().unwrap(),)
        } else {
            // Different types are not ordered. Sorry Erlang, it's weird.
            None
        }
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        dispatch!(Debug::fmt, self, f,)
    }
}

impl PrimitiveOperations for Object {
    fn type_name(&self) -> &'static str {
        dispatch!(PrimitiveOperations::type_name, self,)
    }

    fn neg(&self, rt: &mut Runtime) -> Result<Value, Error> {
        dispatch!(PrimitiveOperations::neg, self, rt,)
    }

    fn not(&self, rt: &mut Runtime) -> Result<Value, Error> {
        dispatch!(PrimitiveOperations::not, self, rt,)
    }

    fn add(&self, other: Value, rt: &mut Runtime) -> Result<Value, Error> {
        dispatch!(PrimitiveOperations::add, self, other, rt,)
    }

    fn index(&self, key: Value, rt: &mut Runtime) -> Result<Value, Error> {
        dispatch!(PrimitiveOperations::index, self, key, rt,)
    }

    fn is_truthy(&self) -> bool {
        true
    }

    fn cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(self, other)
    }
}
