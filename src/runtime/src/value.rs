//! Runtime representation of both boxed and unboxed values.
//!
//! We use a technique called NaN-boxing. This takes advantage of how an 64-bit
//! floats are laid out to cram other smaller data into how they represent NaNs.
//!
//! This is done so we can have `f64`s without needing any other type tags or
//! heap allocations, in a dynamically typed context. This is mostly to allow
//! for faster floating point math.
//!
//! In essence, an [`f64`] is like this struct:
//!
//! ``` no_compile
//! struct f64 {
//!     sign: u1,
//!     exponent: i11,
//!     mantissa: i52,
//! }
//! ```
//!
//! When all the bits in the exponent are ones, it indicate a special value, as
//! seen in the table below.
//!
//! ``` text
//! Name  Bits
//!       S|EEEEEEEEEEE|MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM
//! NAN   0|11111111111|1000000000000000000000000000000000000000000000000000
//! INF   0|11111111111|0000000000000000000000000000000000000000000000000000
//! -INF  1|11111111111|0000000000000000000000000000000000000000000000000000
//! ```
//!
//! Especially important is that pointers don't use all 64-bits, due to other
//! architectural limitations on virtual memory size. Both mainstream modern
//! 64-bit architectures (x86_64 and AArch64) give us enough room to work with
//! in practice.
//!
//! We can then take the 3 extra bits between the 13 bits which signal a NAN and
//! the lower 48 bits to tag the type tag the remaining smaller value like a
//! small integer, boolean or even a unicode code point.
//!
//! One important thing to keep in mind here is that `f64`s can do 53-bit
//! integers without a loss of precision, so when we do pack 48-bit integers and
//! natural numbers they're actually smaller than what we could represent with
//! just an `f64`. The reason we _want_ to use these types then should be
//! _because they're never imprecise_, not because they are larger.

use std::ptr::NonNull;

/// A value which is either stored inline or a pointer to a garbage collected
/// [`Managed`] value.
#[derive(Clone)]
pub struct Value(u64);

impl Default for Value {
    fn default() -> Self {
        Value::UNIT
    }
}

impl Value {
    /// If the first 13 bits are all set, then it's packed value.
    const PACKED_MASK: u64 = 0xFFF8_0000_0000_0000;

    /// The bits used by the payload.
    const PAYLOAD_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

    /// The bits used to create a "safe" nan value that will not be interpreted
    /// as a packed value.
    const SAFE_NAN_BITS: u64 = 0x7FF8_0000_0000_0000;

    /// The bits reserved for our type tags in packed values.
    const TAG_BITS_MASK: u64 = 0x0007_0000_0000_0000;

    /// A value which represents the absence of meaningful information. Should
    /// be thought of a zero-element tuple, more than like a `null` value.
    pub const UNIT: Value = Value(Value::PACKED_MASK | Tag::Unit as u64);

    /// The boolean `true`.
    pub const TRUE: Value = Value(Value::PACKED_MASK | Tag::Bool as u64 | 1);

    /// The boolean `false`.
    #[allow(clippy::identity_op)]
    pub const FALSE: Value = Value(Value::PACKED_MASK | Tag::Bool as u64 | 0);

    /// A "safe" non-signaling NaN value.
    pub const NAN: Value = Value(Value::SAFE_NAN_BITS);

    /// The largest valid natural number value that can be stored inline. The
    /// smallest is `0`.
    pub const MAX_NAT: u64 = 281_474_976_710_655u64;

    /// The largest valid (signed) integer value that can be stored inline.
    pub const MAX_INT: i64 = 140_737_488_355_327i64;

    /// The smallest valid (signed) integer value that can be stored inline.
    pub const MIN_INT: i64 = -140_737_488_355_328i64;

    /// Do the bits of this value represent some other value packed inside a
    /// NaN, or is it a floating point number?
    const fn is_packed_value(&self) -> bool {
        self.0 & Value::PACKED_MASK == Value::PACKED_MASK
    }
}

impl Value {
    /// Create a new unit.
    pub const fn unit() -> Value {
        Value::UNIT
    }

    /// Store a [`bool`] as a [`Value`].
    #[inline]
    pub const fn bool(b: bool) -> Value {
        Value(Value::PACKED_MASK | Tag::Bool as u64 | if b { 1 } else { 0 })
    }

    /// Store a [`char`] as a Character [`Value`].
    #[inline]
    pub const fn char(c: char) -> Value {
        Value(Value::PACKED_MASK | Tag::Char as u64 | c as u64)
    }

    /// Store a [`u32`] as a Nat [`Value`].
    #[inline]
    pub const fn nat_u32(u: u32) -> Value {
        Value(Value::PACKED_MASK | Tag::Nat as u64 | u as u64)
    }

    /// Store an [`u64`] as an Nat [`Value`] if it fits inside
    /// [`Value::MAX_NAT`].
    pub fn nat(n: u64) -> Option<Value> {
        if n > Value::MAX_NAT {
            eprintln!("{} doesn't fit in a 48-bit unsigned integer.", n);
            None
        } else {
            Some(Value(
                (n & Value::PAYLOAD_MASK)
                    | Value::PACKED_MASK
                    | Tag::Nat as u64,
            ))
        }
    }

    /// Store a [`u32`] as an Integer [`Value`].
    #[inline]
    pub const fn int_u32(u: u32) -> Value {
        Value(Value::PACKED_MASK | Tag::Int as u64 | u as u64)
    }

    /// Store a [`i32`] as an Integer [`Value`].
    #[inline]
    pub const fn int_i32(i: i32) -> Value {
        // We need to do some work to get the right sign bits set for the high
        // payload byte.
        let i = i as i64 as u64 & Value::PAYLOAD_MASK;

        Value(Value::PACKED_MASK | Tag::Int as u64 | i as u64)
    }

    /// Store an [`i64`] as an Integer [`Value`] if it fits inside
    /// [`Value::MAX_INT`] and [`Value::MIN_INT`].
    pub fn int(i: i64) -> Option<Value> {
        if i > Value::MAX_INT || i < Value::MIN_INT {
            eprintln!("i doesn't fit as {} {:x}", i, i);
            None
        } else {
            Some(Value(
                (i as u64 & Value::PAYLOAD_MASK)
                    | Value::PACKED_MASK
                    | Tag::Int as u64,
            ))
        }
    }

    /// Store a [`f64`] as a [`Value`].
    ///
    /// Note that due to how [`Value`] is stored, quiet NaNs are converted into
    /// signaling NaNs.
    ///
    /// This function isn't `const` because some needed parts of [`f64`] aren't
    /// yet either.
    #[inline]
    pub fn float(f: f64) -> Value {
        let bits = if f.is_nan() {
            Value::SAFE_NAN_BITS
        } else {
            f.to_bits()
        };

        Value(bits)
    }
}

impl Value {
    /// Is this value `()`
    pub fn is_unit(&self) -> bool {
        self.0 == Value::UNIT.0
    }

    /// Is this value a Boolean?
    pub fn is_bool(&self) -> bool {
        self.0 & Value::TAG_BITS_MASK == Tag::Bool as u64
    }

    /// Use this value as a Rust [`bool`] if it's a Boolean.
    pub fn as_bool(&self) -> Option<bool> {
        if self.0 == Value::TRUE.0 {
            Some(true)
        } else if self.0 == Value::FALSE.0 {
            Some(false)
        } else {
            None
        }
    }

    /// Is this value a Character?
    pub fn is_char(&self) -> bool {
        self.0 & Value::TAG_BITS_MASK == Tag::Char as u64
    }

    /// Use this value as a Rust [`char`] if it's a Character.
    pub fn as_char(&self) -> Option<char> {
        if self.is_char() {
            char::from_u32(self.0 as u32)
        } else {
            None
        }
    }

    /// Is this value a Natural number?
    fn is_nat(&self) -> bool {
        self.0 & Value::TAG_BITS_MASK == Tag::Nat as u64
    }

    /// Use this value as a Rust [`u64`] if it's an natural number. Note that
    /// this will always be between 0 and [`Value::MAX_NAT`], i.e it must fit in
    /// a 48-bit unsigned value.
    fn as_nat(&self) -> Option<u64> {
        if self.is_nat() {
            // We shift back and forth to get the right sign extension.
            Some(self.0 & Value::PAYLOAD_MASK)
        } else {
            None
        }
    }

    /// Is this value an Integer?
    fn is_int(&self) -> bool {
        self.0 & Value::TAG_BITS_MASK == Tag::Int as u64
    }

    /// Use this value as a Rust [`i64`] if it's an Integer. Note that this will
    /// always be between [`Value::MAX_INT`] and [`Value::MIN_INT`], i.e it must
    /// fit in a 48-bit integer.
    fn as_int(&self) -> Option<i64> {
        if self.is_int() {
            // We shift back and forth to get the right sign extension.
            Some(((self.0 & Value::PAYLOAD_MASK) << 16) as i64 >> 16)
        } else {
            None
        }
    }

    /// Is this value an [`f64`]?
    fn is_float(&self) -> bool {
        !self.is_packed_value()
    }

    /// View this value as a [`f64`] if it is one.
    fn as_float(&self) -> Option<f64> {
        if self.is_float() {
            Some(f64::from_bits(self.0))
        } else {
            None
        }
    }

    /// Is this value a pointer to a garbage collected value?
    ///
    /// If you need to know if the pointer is to a specific managed type, you
    /// want to use [`is_gc`][Value::is_gc].
    pub fn is_any_gc(&self) -> bool {
        self.0 & Value::TAG_BITS_MASK == Tag::GcPtr as u64
    }
}

// Raw pointers details.
//
// This is where the *magic* (i.e. horribly unsafe code) is.
impl Value {
    /// View the packed bits as a raw pointer. Nothing is checked, not even that
    /// the [`Tag`] indicates this should be used as pointer.
    ///
    /// # Safety
    ///
    /// All safety consideration for using this must be made by the caller.
    /// Nothing is guaranteed.
    const unsafe fn as_raw_ptr_unchecked<T>(&self) -> *mut T {
        let bits: usize;

        #[cfg(target_arch = "x86_64")]
        {
            bits = Value::bits_to_ptr_x86_64(self.0 as usize);
        }

        #[cfg(target_arch = "aarch64")]
        {
            bits = Value::bits_to_ptr_aarch64(self.0 as usize);
        }

        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
        {
            std::compile_error!("Your target_arch isn't supported!")
        }

        bits as _
    }

    // Intel 64 Software Developer's Manual 3.3.7.1 on Canonical Addressing
    // says:
    //
    // > The first implementation of IA-32 processors with Intel 64 architecture
    // > supports a 48-bit linear address. This means a canonical address must
    // > have bits 63 through 48 set to zeros or ones (depending on whether bit
    // > 47 is a zero or one).
    //
    // Note that they count bits from 0–63, so it's the high 16 bits that need
    // to be a sign carry for the high bit of the 48 used by the pointer.
    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    const unsafe fn bits_to_ptr_x86_64(bits: usize) -> usize {
        // arithmetic right shift is used for signed numbers, so we do some
        // casting.
        (((bits << 16) as isize) >> 16) as usize
    }

    // Arm Architecture Reference Manual Armv8 D5.1.3 says there are a few
    // options for different arm implementations: 48 bits, 52 bits and 64 bits,
    // with any high unused bits set to zero.
    //
    // While more bits are possible, I think all we can do at this point is plan
    // for 48, and hope it works out. If this does become a problem on future
    // devices we can try to restrict this to 48-bits in the allocator.
    //
    // On Linux at least, even where 52 bits is supported, "the kernel will, by
    // default, return virtual addresses to user-space from a 48-bit range."
    //
    // https://www.kernel.org/doc/html/latest/arm64/memory.html
    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    const unsafe fn bits_to_ptr_aarch64(bits: usize) -> usize {
        bits & 0x0000_FFFF_FFFF_FFFF
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        // If they're both floats, we compare as floats.
        //
        // If they're packed [`Gc`] pointers, we defer to the [`Object`]s to do
        // equality.
        //
        // If the tag bits don't match bitwise, then they're different types and
        // shouldn't match.
        //
        // All non-pointer types are [`Copy`], so we can just compare bits for
        // comparing payloads.
        if self.is_float() && other.is_float() {
            self.as_float() == other.as_float()
        } else if self.is_any_gc() && other.is_any_gc() {
            unimplemented!("gc types don't have polymorphic equality yet")
        } else {
            self.0 == other.0
        }
    }
}

impl Eq for Value {}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == Value::UNIT.0 {
            write!(f, "()")
        } else if let Some(b) = self.as_bool() {
            write!(f, "{}", b)
        } else if let Some(c) = self.as_char() {
            write!(f, "{:?}", c)
        } else if let Some(n) = self.as_nat() {
            write!(f, "{}", n)
        } else if let Some(i) = self.as_int() {
            write!(f, "{}", i)
        } else {
            write!(f, "<ptr: {:x}>", self.0)
        }
    }
}

/// Type tags.
///
/// Note that there isn't a tag for floating point values because floats are
/// assumed for any non-NaN (and some NaN) values.
#[repr(u64)]
enum Tag {
    Unit = 0x0000_0000_0000_0000,
    Bool = 0x0001_0000_0000_0000,
    Char = 0x0002_0000_0000_0000,
    Nat = 0x0003_0000_0000_0000,
    Int = 0x0004_0000_0000_0000,
    _Reserved0 = 0x0005_0000_0000_0000,
    _Reserved1 = 0x0006_0000_0000_0000,
    GcPtr = 0x0007_0000_0000_0000,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn packing_unit() {
        let unit = Value::UNIT;
        assert!(unit.is_unit());
        assert!(!unit.is_bool());
        assert!(!unit.is_char());
        assert!(!unit.is_int());
        assert!(!unit.is_float());
        assert!(unit.is_packed_value());
    }

    #[test]
    fn packing_bool() {
        assert_eq!(Value::bool(true).0, Value::TRUE.0);
        assert_eq!(Value::bool(false).0, Value::FALSE.0);

        assert!(Value::TRUE.is_bool());
        assert!(Value::FALSE.is_bool());

        assert_eq!(Value::TRUE.as_float(), None);
        assert_eq!(Value::TRUE.as_bool(), Some(true));
    }

    #[test]
    fn packing_char() {
        let a = Value::char('a');
        assert!(a.is_char());
        assert_eq!(a.as_char(), Some('a'));

        let emoji = Value::char('🥳');
        assert!(emoji.is_char());
        assert_eq!(emoji.as_char(), Some('🥳'));

        assert_eq!(a.as_float(), None);
        assert_eq!(a.as_bool(), None);
    }

    #[test]
    fn packing_i32() {
        let small = Value::int_u32(123456);
        assert_eq!(small.as_int(), Some(123456));

        let negative = Value::int_i32(-987654);
        assert_eq!(negative.as_int(), Some(-987654));
    }

    #[test]
    fn packing_nat_max() {
        let large = Value::nat(Value::MAX_NAT).unwrap();
        assert_eq!(large.as_nat(), Some(Value::MAX_NAT));
    }

    #[test]
    fn packing_int_max() {
        let large = Value::int(Value::MAX_INT).unwrap();
        assert_eq!(large.as_int(), Some(Value::MAX_INT));
    }

    #[test]
    fn packing_int_min() {
        let negative = Value::int(Value::MIN_INT).unwrap();
        assert_eq!(negative.as_int(), Some(Value::MIN_INT));
    }

    #[test]
    fn float_simple() {
        let f = -123e4;
        assert!(Value::float(f).is_float());
        assert_eq!(Value::float(f).as_float().unwrap().to_bits(), f.to_bits());
    }

    #[test]
    fn float_nan() {
        let f = f64::NAN;
        assert!(Value::float(f).is_float());
        assert!(Value::float(f).as_float().unwrap().is_nan());
    }

    #[test]
    fn float_inf() {
        let f = f64::INFINITY;
        assert!(Value::float(f).is_float());
        assert!(Value::float(f).as_float().unwrap().is_infinite());
    }

    #[test]
    fn float_neg_inf() {
        let f = f64::NEG_INFINITY;
        assert!(Value::float(f).is_float());
        assert!(Value::float(f).as_float().unwrap().is_infinite());
    }

    #[test]
    fn pointer_size_sanity_check() {
        assert_eq!(std::mem::size_of::<u64>(), std::mem::size_of::<usize>());
    }

    #[test]
    fn on_supported_target_arch() {
        assert!(cfg!(any(target_arch = "aarch64", target_arch = "x86_64")))
    }
}
