//! Basic math (and logic) operations.
//!
//! We don't implement Rust's `std::ops` and instead define methods because
//! eventually some of these will need an extra argument for an allocator (say +
//! on strings).
//!
//! Since PartialEq and PartialOrd don't need to allocate, we used those.
//!
//! We'll want to come back in and expand on the [`Error`] type here once we
//! have runtime support for handling these types of errors.

use crate::value::{InlineType, Value};

#[derive(Debug)]
pub enum Error {
    Overflow,
    OverflowOrDivByZero,
    OperationNotSupported,
}

impl Value {
    pub fn not_supported(&self, _: &Value) -> Result<Value, Error> {
        Err(Error::OperationNotSupported)
    }

    pub fn not(&self) -> Result<Value, Error> {
        if let Some(b) = self.as_bool() {
            Ok(Value::bool(!b))
        } else if let Some(n) = self.as_nat() {
            Ok(Value::nat(!n & Value::MAX_NAT).unwrap())
        } else {
            Err(Error::OperationNotSupported)
        }
    }

    pub fn neg(&self) -> Result<Value, Error> {
        match self.inline_type() {
            Some(InlineType::Int(i)) => {
                i.checked_neg().and_then(Value::int).ok_or(Error::Overflow)
            }
            Some(InlineType::Nat(n)) => {
                if (n as i64) < 0 {
                    Err(Error::Overflow)
                } else {
                    (n as i64)
                        .checked_neg()
                        .and_then(Value::int)
                        .ok_or(Error::Overflow)
                }
            }
            Some(InlineType::Float(f)) => Ok(Value::float(-f)),
            _ => Err(Error::OperationNotSupported),
        }
    }

    pub fn add(&self, rhs: &Value) -> Result<Value, Error> {
        match (self.inline_type(), rhs.inline_type()) {
            (Some(InlineType::Nat(a)), Some(InlineType::Nat(b))) => {
                a.checked_add(b).and_then(Value::nat).ok_or(Error::Overflow)
            }

            (Some(InlineType::Int(a)), Some(InlineType::Int(b))) => {
                a.checked_add(b).and_then(Value::int).ok_or(Error::Overflow)
            }

            (Some(InlineType::Float(a)), Some(InlineType::Float(b))) => {
                Ok(Value::float(a + b))
            }

            (_, _) => Err(Error::OperationNotSupported),
        }
    }

    pub fn sub(&self, rhs: &Value) -> Result<Value, Error> {
        match (self.inline_type(), rhs.inline_type()) {
            (Some(InlineType::Nat(a)), Some(InlineType::Nat(b))) => {
                a.checked_sub(b).and_then(Value::nat).ok_or(Error::Overflow)
            }

            (Some(InlineType::Int(a)), Some(InlineType::Int(b))) => {
                a.checked_sub(b).and_then(Value::int).ok_or(Error::Overflow)
            }

            (Some(InlineType::Float(a)), Some(InlineType::Float(b))) => {
                Ok(Value::float(a - b))
            }

            (_, _) => Err(Error::OperationNotSupported),
        }
    }

    pub fn mul(&self, rhs: &Value) -> Result<Value, Error> {
        match (self.inline_type(), rhs.inline_type()) {
            (Some(InlineType::Nat(a)), Some(InlineType::Nat(b))) => {
                a.checked_mul(b).and_then(Value::nat).ok_or(Error::Overflow)
            }

            (Some(InlineType::Int(a)), Some(InlineType::Int(b))) => {
                a.checked_mul(b).and_then(Value::int).ok_or(Error::Overflow)
            }

            (Some(InlineType::Float(a)), Some(InlineType::Float(b))) => {
                Ok(Value::float(a * b))
            }

            (_, _) => Err(Error::OperationNotSupported),
        }
    }

    pub fn div(&self, rhs: &Value) -> Result<Value, Error> {
        match (self.inline_type(), rhs.inline_type()) {
            (Some(InlineType::Nat(a)), Some(InlineType::Nat(b))) => a
                .checked_div(b)
                .and_then(Value::nat)
                .ok_or(Error::OverflowOrDivByZero),

            (Some(InlineType::Int(a)), Some(InlineType::Int(b))) => a
                .checked_div(b)
                .and_then(Value::int)
                .ok_or(Error::OverflowOrDivByZero),

            (Some(InlineType::Float(a)), Some(InlineType::Float(b))) => {
                Ok(Value::float(a / b))
            }

            (_, _) => Err(Error::OperationNotSupported),
        }
    }

    pub fn pow(&self, rhs: &Value) -> Result<Value, Error> {
        match (self.inline_type(), rhs.inline_type()) {
            (Some(InlineType::Nat(a)), Some(InlineType::Nat(b))) => a
                .checked_pow(b.try_into().map_err(|_| Error::Overflow)?)
                .and_then(Value::nat)
                .ok_or(Error::Overflow),

            (Some(InlineType::Int(a)), Some(InlineType::Int(b))) => a
                .checked_pow(b.try_into().map_err(|_| Error::Overflow)?)
                .and_then(Value::int)
                .ok_or(Error::Overflow),

            (Some(InlineType::Float(a)), Some(InlineType::Float(b))) => {
                Ok(Value::float(a.powf(b)))
            }

            (_, _) => Err(Error::OperationNotSupported),
        }
    }

    /// Unlike the others, this method isn't named the same as the corresponding
    /// opcode, or rust trait, as `mod` is a keyword.
    pub fn modulo(&self, rhs: &Value) -> Result<Value, Error> {
        match (self.inline_type(), rhs.inline_type()) {
            (Some(InlineType::Nat(a)), Some(InlineType::Nat(b))) => {
                Value::nat(a % b).ok_or(Error::Overflow)
            }

            (Some(InlineType::Int(a)), Some(InlineType::Int(b))) => {
                Value::int(a % b).ok_or(Error::Overflow)
            }

            (Some(InlineType::Float(a)), Some(InlineType::Float(b))) => {
                Ok(Value::float(a % b))
            }

            (_, _) => Err(Error::OperationNotSupported),
        }
    }

    pub fn bit_and(&self, rhs: &Value) -> Result<Value, Error> {
        match (self.inline_type(), rhs.inline_type()) {
            (Some(InlineType::Bool(a)), Some(InlineType::Bool(b))) => {
                Ok(Value::bool(a && b))
            }

            (Some(InlineType::Nat(a)), Some(InlineType::Nat(b))) => {
                Ok(Value::nat(a & b).unwrap())
            }

            (Some(InlineType::Int(a)), Some(InlineType::Int(b))) => {
                Ok(Value::int(a & b).unwrap())
            }

            (_, _) => Err(Error::OperationNotSupported),
        }
    }

    pub fn bit_or(&self, rhs: &Value) -> Result<Value, Error> {
        match (self.inline_type(), rhs.inline_type()) {
            (Some(InlineType::Bool(a)), Some(InlineType::Bool(b))) => {
                Ok(Value::bool(a || b))
            }

            (Some(InlineType::Nat(a)), Some(InlineType::Nat(b))) => {
                Value::nat(a | b).ok_or(Error::Overflow)
            }

            (Some(InlineType::Int(a)), Some(InlineType::Int(b))) => {
                Value::int(a | b).ok_or(Error::Overflow)
            }

            (_, _) => Err(Error::OperationNotSupported),
        }
    }

    pub fn bit_xor(&self, rhs: &Value) -> Result<Value, Error> {
        match (self.inline_type(), rhs.inline_type()) {
            (Some(InlineType::Bool(a)), Some(InlineType::Bool(b))) => {
                Ok(Value::bool(a != b))
            }

            (Some(InlineType::Nat(a)), Some(InlineType::Nat(b))) => {
                Value::nat((a ^ b) & Value::PAYLOAD_MASK).ok_or(Error::Overflow)
            }

            (Some(InlineType::Int(a)), Some(InlineType::Int(b))) => {
                Value::int((a ^ b) & Value::PAYLOAD_MASK as i64)
                    .ok_or(Error::Overflow)
            }

            (_, _) => Err(Error::OperationNotSupported),
        }
    }

    pub fn bit_shl(&self, rhs: &Value) -> Result<Value, Error> {
        match (self.inline_type(), rhs.inline_type()) {
            (Some(InlineType::Nat(a)), Some(InlineType::Nat(b))) => {
                Value::nat((a << b) & Value::PAYLOAD_MASK)
                    .ok_or(Error::Overflow)
            }

            (Some(InlineType::Int(a)), Some(InlineType::Int(b))) => {
                Value::int(a << b).ok_or(Error::Overflow)
            }

            (_, _) => Err(Error::OperationNotSupported),
        }
    }

    pub fn bit_shr(&self, rhs: &Value) -> Result<Value, Error> {
        match (self.inline_type(), rhs.inline_type()) {
            (Some(InlineType::Nat(a)), Some(InlineType::Nat(b))) => {
                Value::nat((a >> b) & Value::MAX_NAT).ok_or(Error::Overflow)
            }

            (_, _) => Err(Error::OperationNotSupported),
        }
    }

    pub fn bit_sha(&self, rhs: &Value) -> Result<Value, Error> {
        match (self.inline_type(), rhs.inline_type()) {
            (Some(InlineType::Int(a)), Some(InlineType::Int(b))) => {
                Value::int(a >> b).ok_or(Error::Overflow)
            }

            (_, _) => Err(Error::OperationNotSupported),
        }
    }
}
