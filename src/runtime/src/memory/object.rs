//! Our common base object type that all run time values which allocate are
//! based on.

use std::any::TypeId;

use super::class::Class;

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

    /// The concrete type of the object, it's [`Class`]. This is used to recover
    /// the type of an [`Object`] and safely downcast it.
    concrete_type_id: TypeId,
}

impl Object {
    /// Initialize an the common object fields for some object.
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
            concrete_type_id: TypeId::of::<C>(),
        })
    }

    /// The specific [`Class`] of this object, as initialized.
    pub(crate) fn concrete_type_id(&self) -> TypeId {
        self.concrete_type_id
    }

    /// The size of the object's underlying allocation, in bytes.
    pub(crate) fn size(&self) -> usize {
        self.size
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
}

impl PartialEq for Object {
    fn eq(&self, _other: &Self) -> bool {
        unimplemented!()
    }
}
