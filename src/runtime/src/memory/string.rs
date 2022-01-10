//! Runtime representation of UTF-8 strings.

use std::{alloc::Layout, io::Write, ptr::NonNull};

use crate::memory::{
    allocator::InitWith,
    collector::{Trace, WorkList},
    Header, Managed,
};

/// An immutable UTF-8 string.
#[derive(Default)]
pub struct String {
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

impl Managed for String {}

impl Trace for String {
    fn trace(&mut self, worklist: &mut WorkList) {
        // Nothing ot trace since String doesn't retain any GC pointers.
    }
}

impl InitWith<&str> for String {
    unsafe fn init_with(mut ptr: NonNull<Self>, args: &str) {
        ptr.as_ptr().write(String::default());

        let s = ptr.as_mut();
        s.len = args.len();
        s.push_str_at(0, args);
    }

    fn trailing_bytes_for(args: &&str) -> Option<usize> {
        Some(args.len())
    }
}

impl InitWith<&[&str]> for String {
    unsafe fn init_with(mut ptr: NonNull<Self>, args: &[&str]) {
        ptr.as_ptr().write(String::default());

        let string = ptr.as_mut();
        string.len = args.iter().map(|s| s.len()).sum();

        let mut offset = 0;
        for s in args {
            string.push_str_at(offset, s);
            offset += s.len();
        }
    }

    fn trailing_bytes_for(args: &&[&str]) -> Option<usize> {
        Some(args.iter().map(|s| s.len()).sum())
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

    #[test]
    fn init_with_slice() {
        let mut alloc = SystemAllocator::default();
        let empty = alloc
            .make_with::<String, _>(["hello, ", "world!"].as_slice())
            .unwrap();

        assert!(!empty.is_empty());
        let expected = "hello, world!";
        assert_eq!(empty.len(), expected.len());
        assert_eq!(empty.as_str(), expected);
    }
}
