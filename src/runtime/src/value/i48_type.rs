//! A 48-bit signed integer type.
//!
//! Note that the usual Rust `as` keyword doesn't quite work with user types,
//! and you'll have to use `std::convert` traits instead.
//!
//! With optimizations, these mostly compile away into 64-bit words with a shift
//! here or there to sign-extend.
//!
//! # Safety
//!
//! The 48 bit values are stored as an array (`[u8; 6]`). Operations convert
//! values into [`i64`]s and back as needed.
//!
//! Overflow and underflow are _weird_ and don't wrap around the way you might
//! expect. Operations don't typically check for them.

use std::convert::TryFrom;

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct i48([u8; 6]);

impl i48 {
    /// The largest valid (signed) integer value that can be stored in 48 bits.
    pub const MAX_I64: i64 = 140_737_488_355_327i64;

    /// The smallest valid (signed) integer value that can be stored in 48 bits.
    pub const MIN_I64: i64 = -140_737_488_355_328i64;

    /// The largest integer value that can be stored in an `i48`.
    pub const MAX: i48 = i48::from_i64_unchecked(i48::MAX_I64);

    /// The largest integer value that can be stored in an `i48`.
    pub const ZERO: i48 = i48::from_i64_unchecked(0);

    /// The smallest integer value that can be stored in an `i48`.
    pub const MIN: i48 = i48::from_i64_unchecked(i48::MIN_I64);

    /// Convert an i64 into an i48, blindly chopping off the high bits.
    ///
    /// This _will_ produce wrong results if `i` isn't between [`i48::MAX`] and
    /// [`i48::MIN`]
    #[inline(always)]
    pub const fn from_i64_unchecked(i: i64) -> i48 {
        #[cfg(target_endian = "big")]
        compile_error!("Big endian is not supported");

        let [a, b, c, d, e, f, _, _] = i.to_ne_bytes();

        i48([a, b, c, d, e, f])
    }

    /// Convert an i64 into an i48, blindly chopping off the high bits.
    ///
    /// This _will_ produce wrong results if `i` isn't between [`i48::MAX`] and
    /// [`i48::MIN`]
    #[inline(always)]
    pub const fn from_i64(i: i64) -> Option<i48> {
        if i < i48::MIN_I64 || i > i48::MAX_I64 {
            None
        } else {
            Some(i48::from_i64_unchecked(i))
        }
    }

    /// Convert an i48 into an i64.
    ///
    /// This can't fail.
    #[inline(always)]
    pub const fn as_i64(self) -> i64 {
        let [a, b, c, d, e, f] = self.0;

        let bits = u64::from_ne_bytes([a, b, c, d, e, f, 0, 0]);

        // to sign extend the bits
        (bits << 16) as i64 >> 16
    }

    /// Powers
    #[inline(always)]
    pub const fn pow(self, rhs: i48) -> i48 {
        i48::from_i64_unchecked(self.as_i64().pow(rhs.as_i64() as u32))
    }
}

impl std::fmt::Debug for i48 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", (*self).as_i64())
    }
}

impl std::fmt::Display for i48 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", (*self).as_i64())
    }
}

impl Default for i48 {
    fn default() -> Self {
        i48::from_i64_unchecked(0)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TryFromIntError {
    NegOverflow,
    PosOverflow,
}

impl Into<i64> for i48 {
    fn into(self) -> i64 {
        self.as_i64()
    }
}

impl TryFrom<i64> for i48 {
    // true for overflow
    type Error = TryFromIntError;

    fn try_from(i: i64) -> Result<Self, Self::Error> {
        if i > i48::MAX_I64 {
            Err(TryFromIntError::PosOverflow)
        } else if i < i48::MIN_I64 {
            Err(TryFromIntError::NegOverflow)
        } else {
            Ok(i48::from_i64_unchecked(i))
        }
    }
}

impl PartialEq for i48 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for i48 {}

impl PartialOrd for i48 {
    fn partial_cmp(&self, other: &i48) -> Option<std::cmp::Ordering> {
        self.as_i64().partial_cmp(&other.as_i64())
    }
}

impl Ord for i48 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_i64().cmp(&other.as_i64())
    }
}

macro_rules! impl_from {
    ($t: ty) => {
        impl From<$t> for i48 {
            fn from(n: $t) -> Self {
                i48::from_i64_unchecked(n as i64)
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
        impl $op for i48 {
            type Output = i48;

            fn $name(self) -> Self::Output {
                i48::from_i64_unchecked(self.as_i64().$name())
            }
        }
    };

    (2, $op: path, $name: ident) => {
        impl $op for i48 {
            type Output = i48;

            fn $name(self, other: Self) -> Self::Output {
                let lhs = self.as_i64();
                let rhs = other.as_i64();
                i48::from_i64_unchecked(lhs.$name(rhs))
            }
        }
    };
}

math_op!(1, std::ops::Neg, neg);
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
        for i in &[i48::MAX_I64, 0, i48::MIN_I64, 58234579, -63457268] {
            let a = i48::from_i64_unchecked(*i);
            let b = a.as_i64();
            assert_eq!(*i, b, "expected {:x} to equal {:x}", *i, b);
        }
    }

    #[test]
    fn math() {
        assert_eq!((i48::from(48) + i48::from(16)).as_i64(), 64);
    }
}
