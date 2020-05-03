//! The `Value` enum, a loosely typed way of representing any RON value.

use std::{
    cmp::Ordering,
    convert::TryInto,
    hash::{self, Hash},
};

#[test]
fn value_structure_sizes() {
    assert_eq!(std::mem::size_of::<Value>(), 32);
    assert_eq!(std::mem::size_of::<Struct>(), 24);
    assert_eq!(std::mem::size_of::<Map<Value, Value>>(), 8);
    assert_eq!(std::mem::size_of::<Fields>(), 32);
    assert_eq!(std::mem::size_of::<Integer>(), 16);
    assert_eq!(std::mem::size_of::<Float>(), 8);
}

/// An arbitrary value in a RON document.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Value {
    // compound types
    /// A heterogeneous record, written with `( ... )`.
    ///
    /// Corresponds to the `serde` types `option`, `unit`, `unit_struct`,
    /// `unit_variant`, `newtype_struct`, `newtype_variant`, `tuple`,
    /// `tuple_struct`, `tuple_variant`, `struct`, and `struct_variant`.
    Struct(Struct),
    /// A dynamic record, written with `{ ... }`.
    ///
    /// Note that this library does not actually enforce that map keys and
    /// values are homogeneous, but this is still required for well-formed RON.
    ///
    /// Corresponds to the `serde` type `map`.
    Map(Map<Value, Value>),
    /// A homogeneous record, written with `[ ... ]`.
    ///
    /// Note that this library does not actually enforce that arrays are
    /// homogeneous, but this is still required for well-formed RON.
    ///
    /// Corresponds to the `serde` type `seq`.
    Array(Vec<Value>),

    // primitive types
    /// A UTF-8 sequence, written with `" ... "`.
    ///
    /// Corresponds to the `serde` type `string`.
    String(Box<str>),
    /// A byte sequence, written with `b" ... "` (base64).
    ///
    /// Corresponds to the `serde` type `byte array`.
    Bytes(Vec<u8>),
    /// A boolean value, `true` or `false`.
    ///
    /// Corresponds to the `serde` type `bool`.
    Bool(bool),
    /// A signed integer.
    ///
    /// Corresponds to the `serde` types `i8`, `i16`, `i32`, `i64`, and `i128`.
    Signed(Sign, Integer),
    /// An unsigned integer.
    ///
    /// Corresponds to the `serde` types `u8`, `u16`, `u32`, `u64` and `u128`.
    Unsigned(Integer),
    /// A floating point number.
    ///
    /// Corresponds to the `serde` types `f32` and `f64`.
    Float(Float),
    /// A single unicode codepoint, written with `' ... '`.
    ///
    /// Corresponds to the `serde` type `char`.
    Char(char),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
/// A heterogeneous record, written with `( ... )`.
///
/// Corresponds to the `serde` types `option`, `unit`, `unit_struct`,
/// `unit_variant`, `newtype_struct`, `newtype_variant`, `tuple`,
/// `tuple_struct`, `tuple_variant`, `struct`, and `struct_variant`.
pub struct Struct {
    /// The (optional) name of the record.
    pub name: Option<Box<str>>,
    /// The (optional) fields of the record.
    pub fields: Option<Box<Fields>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Fields {
    Named(Map<Box<str>, Value>),
    Unnamed(Vec<Value>),
}

/// A dynamic record, written with `{ ... }`.
///
/// Note that this library does not actually enforce that map keys and
/// values are homogeneous, but this is still required for well-formed RON.
///
/// Corresponds to the `serde` type `map`.
#[derive(Debug, Clone)]
pub struct Map<Key: Eq + Hash, Val> {
    raw: Box<indexmap::IndexMap<Key, Val>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Sign {
    Positive,
    Negative,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Integer {
    raw: u128,
}

#[derive(Debug, Copy, Clone)]
pub struct Float {
    raw: f64,
}

// indexmap::IndexMap doesn't provide Hash
// also manually implement Partial/Eq to match Hash
impl<Key: Eq + Hash, Val: Eq> Eq for Map<Key, Val> {}
impl<Key: Eq + Hash, Val: PartialEq> PartialEq for Map<Key, Val> {
    fn eq(&self, other: &Self) -> bool {
        self.raw.iter().eq(other.raw.iter())
    }
}
impl<Key: Eq + Hash, Val: Hash> Hash for Map<Key, Val> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.raw.iter().for_each(|x| x.hash(state))
    }
}

// IEEE wizardry; comparison as signed numbers is still meaningful for non-NaN
impl Eq for Float {}
impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        self.raw.to_bits() == other.raw.to_bits()
    }
}
impl PartialOrd for Float {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Float {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.raw.to_bits() as i64).cmp(&(other.raw.to_bits() as i64))
    }
}
impl Hash for Float {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write_i64(self.raw.to_bits() as i64)
    }
}

// conversions for primitives
mod sealed {
    #![allow(non_camel_case_types)]
    use super::*;

    pub trait uint: Copy {
        fn make_opaque(i: Self) -> Integer;
    }

    pub trait float: Copy {
        fn make_opaque(i: Self) -> Float;
    }

    macro_rules! impl_uint {
        ($($t:ty)*) => {$(
            impl uint for $t {
                fn make_opaque(i: Self) -> Integer {
                    Integer { raw: i.into() }
                }
            }
        )*};
    }

    macro_rules! impl_float {
        ($($t:ty)*) => {$(
            impl float for $t {
                fn make_opaque(f: Self) -> Float {
                    Float { raw: f.into() }
                }
            }
        )*};
    }

    impl_uint![u8 u16 u32 u64 u128];
    impl_float![f32 f64];
}

#[allow(non_camel_case_types)]
impl<uint: sealed::uint> From<uint> for Integer {
    fn from(i: uint) -> Self {
        uint::make_opaque(i)
    }
}

#[allow(non_camel_case_types)]
impl<float: sealed::float> From<float> for Float {
    fn from(f: float) -> Self {
        float::make_opaque(f)
    }
}

impl Integer {
    pub fn as_u64(self) -> Option<u64> {
        self.raw.try_into().ok()
    }

    pub fn as_u128(self) -> Option<u128> {
        self.raw.try_into().ok()
    }
}

impl Float {
    pub fn as_f32(self) -> f32 {
        self.raw as _
    }

    pub fn as_f64(self) -> f64 {
        self.raw as _
    }
}

// convenience accessors for Value
impl Value {
    pub fn as_struct(&self) -> Option<&Struct> {
        match self {
            Value::Struct(this) => Some(this),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&Map<Value, Value>> {
        match self {
            Value::Map(this) => Some(this),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(this) => Some(this),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(this) => Some(this),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Value::Bytes(this) => Some(this),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Value::Bool(this) => Some(this),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match *self {
            Value::Signed(Sign::Positive, int) | Value::Unsigned(int) => int.as_u64(),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::Signed(Sign::Positive, int) | Value::Unsigned(int) => {
                let u = int.as_u64()?;
                u.try_into().ok()
            }
            Value::Signed(Sign::Negative, int) => {
                match int.as_u64()? {
                    u @ 0..=I64_MAX_AS_U64 => Some(-(u as i64)),
                    u @ I64_MIN_AS_U64 => Some(u as i64),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn as_u128(&self) -> Option<u128> {
        match *self {
            Value::Signed(Sign::Positive, int) => int.as_u128(),
            Value::Unsigned(int) => int.as_u128(),
            _ => None,
        }
    }

    pub fn as_i128(&self) -> Option<i128> {
        match *self {
            Value::Signed(Sign::Positive, int) | Value::Unsigned(int) => {
                let u = int.as_u128()?;
                u.try_into().ok()
            }
            Value::Signed(Sign::Negative, int) => {
                match int.as_u128()? {
                    u @ 0..=I128_MAX_AS_U128 => Some(-(u as i128)),
                    u @ I128_MIN_AS_U128 => Some(u as i128),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        match *self {
            Value::Float(this) => Some(this.as_f32()),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::Float(this) => Some(this.as_f64()),
            _ => None,
        }
    }
}

const I64_MAX_AS_U64: u64 = i64::MAX as u64;
const I64_MIN_AS_U64: u64 = i64::MIN as u64;
const I128_MAX_AS_U128: u128 = i128::MAX as u128;
const I128_MIN_AS_U128: u128 = i128::MIN as u128;
