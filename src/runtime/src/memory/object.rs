//! Our common base object type that all run time values which allocate are
//! based on.

use std::{any::TypeId, cmp::Ordering};

use crate::{
    memory::{
        class::Class,
        collector::GCHeader,
        string::String,
        trace::{Trace, WorkList},
    },
    value::Value,
    Error,
};

use super::{closure::Closure, keyword::Keyword, list::List};

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
/// concrete [`Class`] (using the [`Runtime::make`][crate::Runtime::make])
#[repr(C, align(8))]
pub(crate) struct Object {
    /// The size (in bytes) of the allocation belonging to this [`Object`].
    size: usize,

    /// Data tracked by the garbage collector.
    gc_header: GCHeader,

    /// The concrete type of the object, it's [`Class`]. This is used to recover
    /// the type of an [`Object`] and safely downcast it.
    concrete_type_id: TypeId,
}

impl Object {
    /// The alignment used for all objects.
    pub const ALIGN: usize = 8; // Must keep in sync with repr directive.

    /// The specific [`Class`] of this object, as initialized.
    pub(crate) fn concrete_type_id(&self) -> TypeId {
        self.concrete_type_id
    }

    /// Attempt to cast the object as an reference to a specific [`Class`].
    ///
    /// This return's `None` if the object is not the right class.
    pub fn downcast<C: Class>(&self) -> Option<&C> {
        if self.concrete_type_id() == TypeId::of::<C>() {
            Some(unsafe { std::mem::transmute::<_, _>(self) })
        } else {
            None
        }
    }

    pub fn enqueue_gc_references(&self, worklist: &mut WorkList) {
        match self.concrete_type_id {
            id if id == TypeId::of::<String>() => {
                let string = self.downcast::<String>().unwrap();
                string.enqueue_gc_references(worklist)
            }

            id if id == TypeId::of::<Keyword>() => {
                let keyword = self.downcast::<Keyword>().unwrap();
                keyword.enqueue_gc_references(worklist)
            }

            id if id == TypeId::of::<Closure>() => {
                let closure = self.downcast::<Closure>().unwrap();
                closure.enqueue_gc_references(worklist)
            }

            id if id == TypeId::of::<List>() => {
                let list = self.downcast::<List>().unwrap();
                list.enqueue_gc_references(worklist)
            }

            other => {
                panic!("object with unknown class {:?}", other);
            }
        }
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
            concrete_type_id: TypeId::of::<C>(),
        })
    }

    /// The size of the object's underlying allocation, in bytes.
    pub(crate) fn size(&self) -> usize {
        self.size
    }
}

impl PartialEq for Object {
    fn eq(&self, _other: &Self) -> bool {
        unimplemented!("object equality is not yet implemented")
    }
}

impl PartialOrd for Object {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        unimplemented!("object equality is not yet implemented")
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

        Err(Error::CastError)
    }
}
