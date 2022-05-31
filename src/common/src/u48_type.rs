//! A 48-bit unsigned integer type.
//!
//! Note that the usual Rust `as` keyword doesn't quite work with user types,
//! and you'll have to use `std::convert` traits instead.
//!
//! With optimizations, these mostly compile away into 64-bit words.
//!
//! # Safety
//!
//! The 48 bit values are stored as an array (`[u8; 6]`). Operations convert
//! values into [`u64`]s and back as needed.
//!
//! Overflow is _weird_ and might not wrap around the way you expect. Operations
//! don't typically check, and they're undefined either way.
use std::{convert::TryFrom, num::ParseIntError, str::FromStr};

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct u48([u8; 6]);

impl u48 {
    /// The largest valid (signed) integer value that can be stored in 48 bits.
    pub const MAX_U64: u64 = 281_474_976_710_655u64;

    /// The largest integer value that can be stored in an `u48`.
    pub const MAX: u48 = u48::from_u64_unchecked(u48::MAX_U64);

    /// Convert an i64 into an u48, blindly chopping off the high bits.
    ///
    /// This _will_ produce wrong results if `i` isn't between [`u48::MAX`] and
    /// zero.
    #[inline(always)]
    pub const fn from_u64_unchecked(n: u64) -> u48 {
        #[cfg(target_endian = "big")]
        compile_error!("Big endian is not supported");

        let [a, b, c, d, e, f, _, _] = n.to_ne_bytes();

        u48([a, b, c, d, e, f])
    }

    /// Convert a u64 into a u48, blindly chopping off the high bits.
    ///
    /// This _will_ produce wrong results if `n` is larger than [`u48::MAX`].
    #[inline(always)]
    pub const fn from_u64(n: u64) -> Option<u48> {
        if n > u48::MAX_U64 {
            None
        } else {
            Some(u48::from_u64_unchecked(n))
        }
    }

    /// Convert an u48 into an i64.
    ///
    /// This can't fail.
    #[inline(always)]
    pub const fn as_u64(self: u48) -> u64 {
        let [a, b, c, d, e, f] = self.0;
        u64::from_ne_bytes([a, b, c, d, e, f, 0, 0])
    }

    /// See the documentation for [`from_str_radix`][url] for what this does.
    ///
    /// [url]: https://doc.rust-lang.org/std/primitive.u64.html#method.from_str_radix
    pub fn from_str_radix(
        src: &str,
        radix: u32,
    ) -> Result<Self, ParseIntError> {
        let n64 = u64::from_str_radix(src, radix)?;
        u48::from_u64(n64).ok_or_else(u48::force_pos_overflow_error)
    }

    fn force_pos_overflow_error() -> ParseIntError {
        // we force a positive overflow error, hacky
        unsafe { u8::from_str("256").unwrap_err_unchecked() }
    }
}

impl std::fmt::Debug for u48 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", (*self).as_u64())
    }
}

impl std::fmt::Display for u48 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", (*self).as_u64())
    }
}

impl Default for u48 {
    fn default() -> Self {
        u48::from_u64_unchecked(0)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TryFromIntError {
    NegOverflow,
    PosOverflow,
}

impl From<u48> for u64 {
    fn from(n: u48) -> u64 {
        n.as_u64()
    }
}

impl TryFrom<u64> for u48 {
    // true for overflow
    type Error = TryFromIntError;

    fn try_from(n: u64) -> Result<Self, Self::Error> {
        if n > u48::MAX_U64 {
            Err(TryFromIntError::PosOverflow)
        } else {
            Ok(u48::from_u64_unchecked(n))
        }
    }
}

impl PartialEq for u48 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for u48 {}

impl PartialOrd for u48 {
    fn partial_cmp(&self, other: &u48) -> Option<std::cmp::Ordering> {
        self.as_u64().partial_cmp(&other.as_u64())
    }
}

impl Ord for u48 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_u64().cmp(&other.as_u64())
    }
}

impl FromStr for u48 {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let n: u64 = s.parse()?;

        u48::from_u64(n).ok_or_else(u48::force_pos_overflow_error)
    }
}

macro_rules! impl_from {
    ($t: ty) => {
        impl From<$t> for u48 {
            fn from(n: $t) -> Self {
                u48::from_u64_unchecked(n as u64)
            }
        }
    };
}

impl_from!(bool);
impl_from!(char);
impl_from!(u8);
impl_from!(u16);
impl_from!(u32);
impl_from!(i8);
impl_from!(i16);
impl_from!(i32);

macro_rules! math_op {
    (1, $op: path, $name: ident) => {
        impl $op for u48 {
            type Output = u48;

            fn $name(self) -> Self::Output {
                u48::from_u64_unchecked(self.as_u64().$name())
            }
        }
    };

    (2, $op: path, $name: ident) => {
        impl $op for u48 {
            type Output = u48;

            fn $name(self, other: Self) -> Self::Output {
                let lhs = self.as_u64();
                let rhs = other.as_u64();
                u48::from_u64_unchecked(lhs.$name(rhs))
            }
        }
    };
}

math_op!(1, std::ops::Not, not);
math_op!(2, std::ops::Sub, sub);
math_op!(2, std::ops::Add, add);
math_op!(2, std::ops::Mul, mul);
math_op!(2, std::ops::Div, div);
math_op!(2, std::ops::Rem, rem);
math_op!(2, std::ops::BitAnd, bitand);
math_op!(2, std::ops::BitOr, bitor);
math_op!(2, std::ops::BitXor, bitxor);
math_op!(2, std::ops::Shl, shl);
math_op!(2, std::ops::Shr, shr);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion() {
        for i in &[u48::MAX_U64, 0, 58234579] {
            let a = u48::from_u64_unchecked(*i);
            let b = a.as_u64();
            assert_eq!(*i, b, "expected {:x} to equal {:x}", *i, b);
        }
    }

    #[test]
    fn math() {
        assert_eq!((u48::from(48u32) + u48::from(16u32)).as_u64(), 64);
    }
}
