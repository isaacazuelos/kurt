//! Heap objects.

use super::class::Class;

/// Type tags for all our runtime heap-allocated types.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(usize)]
pub(crate) enum Tag {
    String,
}

/// All our runtime values which live on the heap live as an [`Object`].
///
/// The different types of values all implement [`Class`], and get turned into
/// bytes to be shoved into an object during creation.
#[repr(C, align(8))]
pub(crate) struct Object {
    /// A type tag so we know what kind of object this is for down-casting.
    tag: Tag,

    /// The length (in bytes) of the `data` field.
    size: usize,
}

impl Object {
    pub(crate) unsafe fn init(ptr: *mut Object, tag: Tag, size: usize) {
        ptr.write(Object { tag, size })
    }

    pub(crate) fn tag(&self) -> Tag {
        self.tag
    }

    pub(crate) fn size(&self) -> usize {
        self.size
    }

    pub fn downcast<T: Class>(&self) -> Option<&T> {
        match self.tag() {
            Tag::String => Some(unsafe { std::mem::transmute::<_, _>(self) }),
        }
    }
}

impl PartialEq for Object {
    // Just for now
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}
