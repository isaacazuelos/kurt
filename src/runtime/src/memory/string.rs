//! Runtime representation of UTF-8 strings.

// Right now things is mostly a test of our abstractions. These will get used
// eventually.
#![allow(unused)]

use std::{alloc::Layout, io::Write, ptr::NonNull};

use crate::memory::{Header, Managed};

use super::allocator::InitWith;

/// An immutable UTF-8 string.
pub struct String {
    header: Header,
    len: usize,
    bytes: [u8; 1],
}

impl String {
    /// The length of the string in bytes.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Is this the empty string.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// A view into this string's bytes as a rust `str`.
    pub fn as_str(&self) -> &str {
        unsafe {
            let bytes =
                std::slice::from_raw_parts(self.bytes.as_ptr(), self.len);

            // SAFETY: Strings are always made from `&str`s, which are always
            // valid utf-8.
            std::str::from_utf8_unchecked(bytes)
        }
    }

    /// Push a [`&str`] into this string beginning at the byte `start`,
    /// overwriting the length of it.
    ///
    /// # Safety
    ///
    /// Using this violates the _immutable_ aspect of our string. This only
    /// exists to help initialize the values and must not be called outside of
    /// that context.
    pub(crate) unsafe fn push_str_at(&mut self, start: usize, s: &str) {
        let bytes =
            std::slice::from_raw_parts_mut(self.bytes.as_mut_ptr(), self.len);

        let mut buf = &mut bytes[start..];
        // This must have worked if the caller met our `unsafe` Safety
        // requirements.
        let _ = buf.write(s.as_bytes());
    }
}

impl Default for String {
    fn default() -> String {
        String {
            header: Header::new::<String>(),
            len: 0,
            bytes: [0],
        }
    }
}

unsafe impl Managed for String {}

impl InitWith<&str> for String {
    unsafe fn init_with(mut ptr: NonNull<Self>, args: &str) {
        ptr.as_ptr().write(String::default());

        let s = ptr.as_mut();
        s.len = args.len();
        s.push_str_at(0, args);
    }

    fn layout(args: &&str) -> Option<Layout> {
        let align = std::mem::align_of::<String>();
        let size = std::mem::size_of::<String>() - 1 + args.len();
        Layout::from_size_align(size, align).ok()
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::{allocator::Allocator, system_alloc::SystemAllocator};

    use super::*;

    #[test]
    fn empty() {
        let mut alloc = SystemAllocator::default();
        let empty = alloc.make::<String>().unwrap();

        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
        assert_eq!(empty.as_str(), "");
    }

    #[test]
    fn non_empty() {
        let mut alloc = SystemAllocator::default();
        let empty = alloc.make_with::<String, _>("test").unwrap();

        assert!(!empty.is_empty());
        assert_eq!(empty.len(), 4);
        assert_eq!(empty.as_str(), "test");
    }
}
