use std::{
    fmt::{self, Debug},
    ptr::addr_of_mut,
};

use super::{class::Class, object::Tag, InitFrom, Object};

#[repr(C, align(8))]
pub struct String {
    /// The base object required to be a [`Class`].
    base: Object,

    /// The array that we overflow to store our string, since `String` is
    /// actually a DST in disguise.
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
    const TAG: Tag = Tag::String;
}

impl InitFrom<&str> for String {
    fn size(arg: &&str) -> usize {
        arg.len()
    }

    unsafe fn init(ptr: *mut Self, args: &str) {
        let dst = addr_of_mut!((*ptr).data) as _;
        let src = args.as_ptr();

        std::ptr::copy_nonoverlapping(src, dst, args.len());

        *dst.offset(args.len() as _) = b'\0';
    }
}

impl InitFrom<(&str, &str)> for String {
    fn size(arg: &(&str, &str)) -> usize {
        arg.0.len() + arg.1.len()
    }

    unsafe fn init(ptr: *mut Self, (a, b): (&str, &str)) {
        // copy first string
        let dst = addr_of_mut!((*ptr).data) as _;
        let src = a.as_ptr();
        std::ptr::copy_nonoverlapping(src, dst, a.len());

        // copy second string
        let dst = dst.offset(a.len() as _);
        let src = b.as_ptr();
        std::ptr::copy_nonoverlapping(src, dst, b.len());

        // write null byte
        *dst.offset((a.len() + b.len()) as _) = b'\0';
    }
}

impl String {
    /// The length of the string's contents in bytes.
    ///
    /// This doesn't include the null-terminating byte mentioned in the
    pub fn len(&self) -> usize {
        // We compute it from the base object's allocation size.
        //
        // TODO: We should probably worry about padding here. We just happen not
        //       to have any right now.
        self.upcast().size() - std::mem::size_of::<Object>()
    }

    /// View the underlying UTF-8 bytes of the string as a slice.
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data.as_ptr(), self.len()) }
    }

    /// View this String object's contents as a [`str`].
    pub(crate) fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
    }
}
