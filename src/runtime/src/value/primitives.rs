//! Basic math (and logic) operations.
//!
//! We don't implement Rust's `std::ops` and instead define methods because some
//! of these will need an allocator.
//!
//! Since PartialEq and PartialOrd don't need to allocate, we used those.
//!
//! We'll want to come back in and expand on the [`Error`] type here once we
//! have runtime support for handling these types of errors.

use crate::{
    primitives::{Error, PrimitiveOperations},
    value::Value,
    Runtime,
};

use super::{i48_type::i48, u48_type::u48, Tag};

macro_rules! basic_impl {
    ( $op: path, $t: ty, $lhs: ident, $rhs: ident ) => {{
        let rhs: $t = $rhs.try_into()?;
        Ok(Value::from($op(*$lhs, rhs)))
    }};
}

macro_rules! dispatch {
    ($f: path, $value: ident, $($arg: expr ,)*) => {{
        match $value.tag() {
            Tag::Unit => $f( &(), $( $arg, )* ),
            Tag::Bool => $f( &$value.as_bool().unwrap(), $( $arg, )* ),
            Tag::Char => $f( &$value.as_char().unwrap(), $( $arg, )* ),
            Tag::Float => $f( &$value.as_float().unwrap(), $( $arg, )* ),
            Tag::Int => $f( &$value.as_int().unwrap(), $( $arg, )* ),
            Tag::Nat => $f( &$value.as_nat().unwrap(), $( $arg, )* ),
            Tag::Object => {
                $f( $value.as_object().unwrap().deref(), $( $arg, )* )
            },
            Tag::_Reserved0 | Tag::_Reserved1 => {
                unreachable!("cannot use value with reserved tag")
            }
        }
    }};
}

impl PrimitiveOperations for () {
    fn type_name(&self) -> &'static str {
        "()"
    }

    /// A `()` is seen as false by conditionals.
    fn is_truthy(&self) -> bool {
        false
    }

    fn cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ordering::Equal)
    }
}

impl PrimitiveOperations for bool {
    fn type_name(&self) -> &'static str {
        "Bool"
    }

    fn not(&self, _: &mut Runtime) -> Result<Value, Error> {
        Ok(Value::bool(!self))
    }

    fn bitand(&self, other: Value, _: &mut Runtime) -> Result<Value, Error> {
        basic_impl!(std::ops::BitAnd::bitand, bool, self, other)
    }

    fn bitor(&self, other: Value, _: &mut Runtime) -> Result<Value, Error> {
        basic_impl!(std::ops::BitOr::bitor, bool, self, other)
    }

    fn bitxor(&self, other: Value, _: &mut Runtime) -> Result<Value, Error> {
        basic_impl!(std::ops::BitXor::bitxor, bool, self, other)
    }

    fn is_truthy(&self) -> bool {
        *self
    }

    fn cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(self, other)
    }
}

impl PrimitiveOperations for char {
    fn type_name(&self) -> &'static str {
        "Char"
    }

    /// A Char is truthy if it's not the nul character `'\x{0}'`.
    fn is_truthy(&self) -> bool {
        *self == '\0'
    }

    fn cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(self, other)
    }
}

impl PrimitiveOperations for f64 {
    fn type_name(&self) -> &'static str {
        "Float"
    }

    fn neg(&self, _: &mut Runtime) -> Result<Value, Error> {
        Ok(Value::float(-self))
    }

    fn add(&self, other: Value, _: &mut Runtime) -> Result<Value, Error> {
        basic_impl!(std::ops::Add::add, f64, self, other)
    }

    fn sub(&self, other: Value, _: &mut Runtime) -> Result<Value, Error> {
        basic_impl!(std::ops::Sub::sub, f64, self, other)
    }

    fn mul(&self, other: Value, _: &mut Runtime) -> Result<Value, Error> {
        basic_impl!(std::ops::Mul::mul, f64, self, other)
    }

    fn div(&self, other: Value, _: &mut Runtime) -> Result<Value, Error> {
        basic_impl!(std::ops::Div::div, f64, self, other)
    }

    fn rem(&self, other: Value, _: &mut Runtime) -> Result<Value, Error> {
        basic_impl!(std::ops::Rem::rem, f64, self, other)
    }

    fn is_truthy(&self) -> bool {
        *self == 0.0
    }

    fn cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(self, other)
    }
}

impl PrimitiveOperations for u48 {
    fn type_name(&self) -> &'static str {
        "Nat"
    }
}

impl PrimitiveOperations for i48 {
    fn type_name(&self) -> &'static str {
        "Int"
    }

    fn neg(&self, _: &mut Runtime) -> Result<Value, Error> {
        Ok(Value::int(-*self))
    }

    fn not(&self, _: &mut Runtime) -> Result<Value, Error> {
        Ok(Value::int(!*self))
    }

    fn add(&self, other: Value, _: &mut Runtime) -> Result<Value, Error> {
        basic_impl!(std::ops::Add::add, i48, self, other)
    }

    fn sub(&self, other: Value, _: &mut Runtime) -> Result<Value, Error> {
        basic_impl!(std::ops::Sub::sub, i48, self, other)
    }

    fn mul(&self, other: Value, _: &mut Runtime) -> Result<Value, Error> {
        basic_impl!(std::ops::Mul::mul, i48, self, other)
    }

    fn div(&self, other: Value, _: &mut Runtime) -> Result<Value, Error> {
        basic_impl!(std::ops::Div::div, i48, self, other)
    }

    fn rem(&self, other: Value, _: &mut Runtime) -> Result<Value, Error> {
        basic_impl!(std::ops::Rem::rem, i48, self, other)
    }

    fn pow(&self, other: Value, _: &mut Runtime) -> Result<Value, Error> {
        basic_impl!(i48::pow, i48, self, other)
    }

    fn bitand(&self, other: Value, _: &mut Runtime) -> Result<Value, Error> {
        basic_impl!(std::ops::BitAnd::bitand, i48, self, other)
    }

    fn bitor(&self, other: Value, _: &mut Runtime) -> Result<Value, Error> {
        basic_impl!(std::ops::BitOr::bitor, i48, self, other)
    }

    fn bitxor(&self, other: Value, _: &mut Runtime) -> Result<Value, Error> {
        basic_impl!(std::ops::BitXor::bitxor, i48, self, other)
    }

    fn shl(&self, other: Value, _: &mut Runtime) -> Result<Value, Error> {
        basic_impl!(std::ops::Shl::shl, i48, self, other)
    }

    fn shr(&self, other: Value, _: &mut Runtime) -> Result<Value, Error> {
        basic_impl!(std::ops::Shr::shr, i48, self, other)
    }

    fn cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_i64().partial_cmp(&other.as_i64())
    }
}

impl PrimitiveOperations for Value {
    fn type_name(&self) -> &'static str {
        match self.tag() {
            Tag::Unit => ().type_name(),
            Tag::Bool => true.type_name(),
            Tag::Char => ' '.type_name(),
            Tag::Float => 1f64.type_name(),
            Tag::Int => i48::MAX.type_name(),
            Tag::Nat => u48::MAX.type_name(),
            Tag::Object => self.as_object().unwrap().deref().type_name(),
            Tag::_Reserved0 | Tag::_Reserved1 => "<invalid value>",
        }
    }

    fn neg(&self, rt: &mut Runtime) -> Result<Self, Error> {
        dispatch!(PrimitiveOperations::neg, self, rt,)
    }

    fn not(&self, rt: &mut Runtime) -> Result<Self, Error> {
        dispatch!(PrimitiveOperations::not, self, rt,)
    }

    fn add(&self, other: Value, rt: &mut Runtime) -> Result<Self, Error> {
        dispatch!(PrimitiveOperations::add, self, other, rt,)
    }

    fn sub(&self, other: Value, rt: &mut Runtime) -> Result<Self, Error> {
        dispatch!(PrimitiveOperations::sub, self, other, rt,)
    }

    fn mul(&self, other: Value, rt: &mut Runtime) -> Result<Self, Error> {
        dispatch!(PrimitiveOperations::mul, self, other, rt,)
    }

    fn div(&self, other: Value, rt: &mut Runtime) -> Result<Self, Error> {
        dispatch!(PrimitiveOperations::div, self, other, rt,)
    }

    fn rem(&self, other: Value, rt: &mut Runtime) -> Result<Self, Error> {
        dispatch!(PrimitiveOperations::rem, self, other, rt,)
    }

    fn pow(&self, other: Value, rt: &mut Runtime) -> Result<Self, Error> {
        dispatch!(PrimitiveOperations::pow, self, other, rt,)
    }

    fn bitand(&self, other: Value, rt: &mut Runtime) -> Result<Self, Error> {
        dispatch!(PrimitiveOperations::bitand, self, other, rt,)
    }

    fn bitor(&self, other: Value, rt: &mut Runtime) -> Result<Self, Error> {
        dispatch!(PrimitiveOperations::bitor, self, other, rt,)
    }

    fn bitxor(&self, other: Value, rt: &mut Runtime) -> Result<Self, Error> {
        dispatch!(PrimitiveOperations::bitxor, self, other, rt,)
    }

    fn shl(&self, other: Value, rt: &mut Runtime) -> Result<Self, Error> {
        dispatch!(PrimitiveOperations::shl, self, other, rt,)
    }

    fn shr(&self, other: Value, rt: &mut Runtime) -> Result<Self, Error> {
        dispatch!(PrimitiveOperations::shr, self, other, rt,)
    }

    fn index(&self, other: Value, rt: &mut Runtime) -> Result<Self, Error> {
        dispatch!(PrimitiveOperations::index, self, other, rt,)
    }

    fn is_truthy(&self) -> bool {
        dispatch!(PrimitiveOperations::is_truthy, self,)
    }

    fn cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let tag = self.tag();
        if other.tag() != tag {
            return None;
        }

        match tag {
            Tag::Unit => Some(std::cmp::Ordering::Equal),
            Tag::Bool => self
                .as_bool()
                .unwrap()
                .partial_cmp(&other.as_bool().unwrap()),
            Tag::Char => self
                .as_char()
                .unwrap()
                .partial_cmp(&other.as_char().unwrap()),
            Tag::Nat => {
                self.as_nat().unwrap().partial_cmp(&other.as_nat().unwrap())
            }
            Tag::Int => {
                self.as_int().unwrap().partial_cmp(&other.as_int().unwrap())
            }
            Tag::Float => self
                .as_float()
                .unwrap()
                .partial_cmp(&other.as_float().unwrap()),
            Tag::Object => self
                .as_object()
                .unwrap()
                .deref()
                .partial_cmp(other.as_object().unwrap().deref()),

            _ => None,
        }
    }
}
