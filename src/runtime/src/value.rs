//! Runtime representation of both boxed and unboxed values.
//!
//! We use a technique called NaN-boxing. This takes advantage of how an 64-bit
//! floats are laid out to cram other smaller data into how they represent NaNs.
//!
//! This is done so we can have `f64`s without needing any other type tags or
//! heap allocations, in a dynamically typed context. This is mostly to allow
//! for faster floating point math. I also just think it's neat.
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
//! When all the bits in the exponent are ones, it indicates a special value, as
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
//! small integer, boolean or unicode code point.
//!
//! One important thing to keep in mind here is that `f64`s can do 53-bit
//! integers without a loss of precision, so when we do pack 48-bit integers and
//! natural numbers they're actually smaller than what we could represent with
//! just an `f64`. The reason we _want_ to use these types then should be
//! _because they're never imprecise_, not because they are larger.

use std::ptr::NonNull;

use crate::memory::{
    trace::{Trace, WorkList},
    Gc, Object,
};

/// A value which is either stored inline or as pointer to a garbage collected
/// [`Object`].
///
/// # Note
///
/// Most of the methods are `const` where possible to help indicate if they do
/// anything more than bit twiddling.
#[derive(Clone, Copy)]
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
    pub const PAYLOAD_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

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
    ///
    /// This is the largest 48-bit int.
    pub const MAX_INT: i64 = 140_737_488_355_327i64;

    /// The smallest valid (signed) integer value that can be stored inline.
    ///
    /// This is the smallest 48-bit int.
    pub const MIN_INT: i64 = -140_737_488_355_328i64;

    /// Do the bits of this value represent some other value packed inside a
    /// NaN, or is it a floating point number?
    #[inline(always)]
    const fn is_packed_value(&self) -> bool {
        self.0 & Value::PACKED_MASK == Value::PACKED_MASK
    }
}

impl Value {
    /// Create a new unit.
    #[inline]
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
    #[inline]
    pub const fn nat(n: u64) -> Option<Value> {
        if n > Value::MAX_NAT {
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
    #[inline]
    pub const fn int(i: i64) -> Option<Value> {
        if i > Value::MAX_INT || i < Value::MIN_INT {
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
    #[inline]
    pub fn float(f: f64) -> Value {
        let bits = if f.is_nan() {
            Value::SAFE_NAN_BITS
        } else {
            f.to_bits()
        };

        Value(bits)
    }

    /// Store a [`Gc`] pointer to any [`Object`] as a [`Value`].
    #[inline]
    pub fn object(gc: Gc) -> Value {
        let bits: u64 = unsafe { std::mem::transmute(gc) };

        Value(
            (bits & Value::PAYLOAD_MASK)
                | Value::PACKED_MASK
                | Tag::Object as u64,
        )
    }
}

impl Value {
    /// Is this value `()`
    #[inline]
    pub const fn is_unit(&self) -> bool {
        self.0 == Value::UNIT.0
    }

    /// Is this value a Boolean?
    #[inline]
    pub const fn is_bool(&self) -> bool {
        self.0 & Value::TAG_BITS_MASK == Tag::Bool as u64
    }

    /// Use this value as a Rust [`bool`] if it's a Boolean.
    #[inline]
    pub const fn as_bool(&self) -> Option<bool> {
        if self.0 == Value::TRUE.0 {
            Some(true)
        } else if self.0 == Value::FALSE.0 {
            Some(false)
        } else {
            None
        }
    }

    /// Is this value a Character?
    #[inline]
    pub const fn is_char(&self) -> bool {
        self.0 & Value::TAG_BITS_MASK == Tag::Char as u64
    }

    /// Use this value as a Rust [`char`] if it's a Character.
    #[inline]
    pub fn as_char(&self) -> Option<char> {
        if self.is_char() {
            char::from_u32(self.0 as u32)
        } else {
            None
        }
    }

    /// Is this value a Natural number?
    #[inline]
    pub const fn is_nat(&self) -> bool {
        self.0 & Value::TAG_BITS_MASK == Tag::Nat as u64
    }

    /// Use this value as a Rust [`u64`] if it's an natural number. Note that
    /// this will always be between 0 and [`Value::MAX_NAT`], i.e it must fit in
    /// a 48-bit unsigned value.
    #[inline]
    pub const fn as_nat(&self) -> Option<u64> {
        if self.is_nat() {
            // We shift back and forth to get the right sign extension.
            Some(self.0 & Value::PAYLOAD_MASK)
        } else {
            None
        }
    }

    /// Is this value an Integer?
    #[inline]
    pub const fn is_int(&self) -> bool {
        self.0 & Value::TAG_BITS_MASK == Tag::Int as u64
    }

    /// Use this value as a Rust [`i64`] if it's an Integer. Note that this will
    /// always be between [`Value::MAX_INT`] and [`Value::MIN_INT`], i.e it must
    /// fit in a 48-bit integer.
    #[inline]
    pub const fn as_int(&self) -> Option<i64> {
        if self.is_int() {
            // We shift back and forth to get the right sign extension.
            Some(((self.0 & Value::PAYLOAD_MASK) << 16) as i64 >> 16)
        } else {
            None
        }
    }

    /// Is this value an [`f64`]?
    #[inline]
    pub const fn is_float(&self) -> bool {
        !self.is_packed_value()
    }

    /// View this value as a [`f64`] if it is one.
    #[inline]
    pub fn as_float(&self) -> Option<f64> {
        if self.is_float() {
            Some(f64::from_bits(self.0))
        } else {
            None
        }
    }

    /// Is this value a pointer to a garbage collected value?
    #[inline]
    pub fn is_object(&self) -> bool {
        self.0 & Value::TAG_BITS_MASK == Tag::Object as u64
    }

    /// View this value as an opaque [`Gc`] reference to an [`Object`].
    #[inline]
    pub fn as_object(&self) -> Option<Gc> {
        if self.is_object() {
            unsafe {
                let raw = self.as_raw_ptr_unchecked() as *mut Object;
                let non_null = NonNull::new(raw)?;
                Some(Gc::from_non_null(non_null))
            }
        } else {
            None
        }
    }
}

// Raw pointers details.
//
// This is where the *magic* (i.e. horribly unsafe code) is.
impl Value {
    /// View the packed bits as a raw pointer. Nothing is checked, not even that
    /// the [`Tag`] indicates this should be used as pointer. This has no
    /// guarantees here beyond what you'd expect of any `*mut` pointer.
    ///
    /// Note that this is _not_ really a pointer to a [`u8`], I just need to
    /// give it something so the compiler's happy here, and pointers to
    /// zero-sized types like `()` make me suspicious.
    #[inline(always)]
    const unsafe fn as_raw_ptr_unchecked(&self) -> *mut u8 {
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

impl Trace for Value {
    fn enqueue_gc_references(&self, worklist: &mut WorkList) {
        if let Some(ptr) = self.as_object() {
            worklist.enqueue(ptr);
        }
    }
}

impl PartialEq for Value {
    /// Two values are equal if the unpacked values are equal.
    ///
    /// If they're both floats, we compare as floats.
    ///
    /// We defer to self as an [`Object`] to decide equality if they're both
    /// objects.
    ///
    /// All other inline types are [`Copy`], so we can compare bits. Since the
    /// tag bits won't match if they're different types, so don't need to worry
    /// about the payloads colliding.
    fn eq(&self, other: &Self) -> bool {
        if self.is_float() && other.is_float() {
            self.as_float() == other.as_float()
        } else if self.is_object() && other.is_object() {
            self.as_object()
                .unwrap()
                .deref()
                .eq(other.as_object().unwrap().deref())
        } else {
            self.0 == other.0
        }
    }
}

impl PartialOrd for Value {
    /// We only implement PartialOrd as we don't try to order values of
    /// different types. It also means we don't have to impose a total ordering
    /// onto [`f64`], which doesn't actually have one.
    ///
    /// As with equality, defer to self as an [`Object`] to decide equality if
    /// they're both objects.
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        use InlineType::*;

        if self.is_object() && other.is_object() {
            self.as_object()
                .unwrap()
                .deref()
                .partial_cmp(other.as_object().unwrap().deref())
        } else {
            match (self.inline_type(), other.inline_type()) {
                (Some(Unit), Some(Unit)) => Some(Ordering::Equal),
                (Some(Bool(a)), Some(Bool(b))) => a.partial_cmp(&b),
                (Some(Char(a)), Some(Char(b))) => a.partial_cmp(&b),
                (Some(Nat(a)), Some(Nat(b))) => a.partial_cmp(&b),
                (Some(Int(a)), Some(Int(b))) => a.partial_cmp(&b),
                (Some(Float(a)), Some(Float(b))) => a.partial_cmp(&b),
                _ => None,
            }
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.inline_type() {
            Some(InlineType::Unit) => write!(f, "()"),
            Some(InlineType::Bool(b)) => write!(f, "{}", b),
            Some(InlineType::Char(c)) => write!(f, "{:?}", c),
            Some(InlineType::Nat(u)) => write!(f, "{}", u),
            Some(InlineType::Int(i)) => write!(f, "{}", i),
            Some(InlineType::Float(n)) => write!(f, "{}", n),
            None => {
                if let Some(obj) = self.as_object() {
                    write!(f, "{:?}", obj.deref())
                } else {
                    write!(f, "<unknown: {:x}>", self.0)
                }
            }
        }
    }
}

/// Type tags.
///
/// Note that there isn't a tag for floating point values because floats are
/// assumed for any non-NaN (and some NaN) values.
///
/// These tags must all fit in the 3 bits between the bits which signal a NaN
/// and the 48 bits we use for our payloads, which is why we can only have 8
/// types tagged this way.
///
/// If we need more later, we can merge types smaller than 48 bits (like Bool,
/// Unit, Char) to a single 'Small' tag and use the bits in the third byte
/// instead of the second to further differentiate.
#[repr(u64)]
enum Tag {
    Unit = 0x0000_0000_0000_0000,
    Bool = 0x0001_0000_0000_0000,
    Char = 0x0002_0000_0000_0000,
    Nat = 0x0003_0000_0000_0000,
    Int = 0x0004_0000_0000_0000,
    _Reserved0 = 0x0005_0000_0000_0000,
    _Reserved1 = 0x0006_0000_0000_0000,
    Object = 0x0007_0000_0000_0000,
}

pub enum InlineType {
    Unit,
    Bool(bool),
    Char(char),
    Nat(u64),
    Int(i64),
    Float(f64),
}

impl Value {
    /// Get the type of this value as an inline type
    pub fn inline_type(self) -> Option<InlineType> {
        if self.is_float() {
            return Some(InlineType::Float(self.as_float().unwrap()));
        }

        let tag_bits = self.0 & Value::TAG_BITS_MASK;

        match tag_bits {
            bits if bits == Tag::Unit as _ => Some(InlineType::Unit),
            bits if bits == Tag::Bool as _ => {
                self.as_bool().map(InlineType::Bool)
            }
            bits if bits == Tag::Char as _ => {
                self.as_char().map(InlineType::Char)
            }
            bits if bits == Tag::Nat as _ => self.as_nat().map(InlineType::Nat),
            bits if bits == Tag::Int as _ => self.as_int().map(InlineType::Int),
            _ => None,
        }
    }
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
