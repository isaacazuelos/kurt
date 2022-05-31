use std::{
    fmt::{self, Debug},
    ptr::addr_of_mut,
};

use crate::{memory::*, primitives::PrimitiveOperations};

#[repr(C, align(8))]
pub struct String {
    /// The base object required to be a [`Class`].
    base: Object,

    /// The array that we overflow to store our string, since `String` is
    /// actually a DST in disguise. This is actually the start of an array
    /// that's much longer which can be calculated with [`Self::len`].
    ///
    /// This one byte serves two purposes:
    ///
    /// 1. It prevents rust form getting mad at us for taking the address of the
    ///    ZST `[u8; 0]`.
    ///
    /// 2. When we add the `len` of the str we want to add, it adds an extra
    ///    byte we can use for a `b'\0'` at the end so we can turn this into a C
    ///    char* for free.
    data: [u8; 1],
}

impl Debug for String {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Class for String {
    const ID: ClassId = ClassId::String;
}

impl PrimitiveOperations for String {
    fn type_name(&self) -> &'static str {
        "String"
    }
}

impl PartialOrd for String {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl PartialEq for String {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Trace for String {
    fn enqueue_gc_references(&self, _: &mut WorkList) {
        // strings have no references to other objects
    }
}

impl InitFrom<&str> for String {
    fn extra_size(arg: &&str) -> usize {
        arg.len()
    }

    unsafe fn init(ptr: *mut Self, args: &str) {
        let dst = addr_of_mut!((*ptr).data) as _;
        let src = args.as_ptr();

        // SAFETY: We know it'll fit from the requirements of InitFrom::init. We
        //         know they don't overlap because the non-object fields are
        //         uninitialized.
        std::ptr::copy_nonoverlapping(src, dst, args.len());

        // Our strings know their lengths, but also are null-terminated so we
        // can use them as C-strings. See the note on the `data` field.
        *dst.add(args.len()) = b'\0';
    }
}

impl InitFrom<(&str, &str)> for String {
    fn extra_size(arg: &(&str, &str)) -> usize {
        arg.0.len() + arg.1.len()
    }

    unsafe fn init(ptr: *mut Self, (a, b): (&str, &str)) {
        // SAFETY: See the notes on the InitFrom<&str>::init impl for some of
        //         the safety arguments, since these are pretty similar.

        // copy first string
        let dst = addr_of_mut!((*ptr).data) as _;
        let src = a.as_ptr();
        std::ptr::copy_nonoverlapping(src, dst, a.len());

        // copy second string
        let dst = dst.add(a.len());
        let src = b.as_ptr();
        std::ptr::copy_nonoverlapping(src, dst, b.len());

        // write null byte
        *dst.add(a.len() + b.len()) = b'\0';
    }
}

impl String {
    /// The length of the string's contents in bytes.
    ///
    /// This doesn't include the null-terminating byte mentioned in the docs for
    /// [`String`] itself.
    pub fn len(&self) -> usize {
        // We compute it from the base object's allocation size. This saves us a
        // word compared to tracking the in String too
        self.base.size() - std::mem::size_of::<String>()
    }

    /// Is this an empty string?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// View the underlying UTF-8 bytes of the string as a slice.
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: Doing this is exactly what we created the `data` for. See the
        //         note there too.
        unsafe { std::slice::from_raw_parts(self.data.as_ptr(), self.len()) }
    }

    /// View this String object's contents as a [`str`].
    pub fn as_str(&self) -> &str {
        // SAFETY: We know the bytes are UTF-8 because all init methods for
        //         string check.
        unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
    }
}

#[cfg(test)]
mod string_tests {
    use super::*;

    #[test]
    fn test_size_empty() {
        let empty_string_extra = String::extra_size(&"");
        assert_eq!(empty_string_extra, 0);
    }
}
