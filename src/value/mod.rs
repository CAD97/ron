//! The `Value` enum, a loosely typed way of representing any RON value.

mod de;
mod ser;
mod fmt;
mod transcode_ser;
mod transcode_de;

use std::{
    cmp::Ordering,
    convert::TryInto,
    hash::{self, Hash},
    iter::FromIterator,
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
#[derive(Clone, Eq, PartialEq, Hash)]
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

#[derive(Clone, Eq, PartialEq, Hash)]
/// A heterogeneous record, written with `( ... )`.
///
/// Corresponds to the `serde` types `option`, `unit`, `unit_struct`,
/// `unit_variant`, `newtype_struct`, `newtype_variant`, `tuple`,
/// `tuple_struct`, `tuple_variant`, `struct`, and `struct_variant`.
pub struct Struct {
    /// The (optional) name of the record.
    //  This has to be &'static for serde :(
    pub name: Option<&'static str>,
    /// The (optional) fields of the record.
    pub fields: Option<Box<Fields>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Fields {
    // This has to be &'static for serde :(
    Named(Map<&'static str, Value>),
    Unnamed(Vec<Value>),
}

impl FromIterator<(&'static str, Value)> for Fields {
    fn from_iter<T: IntoIterator<Item = (&'static str, Value)>>(iter: T) -> Self {
        Fields::Named(iter.into_iter().collect())
    }
}

impl FromIterator<Value> for Fields {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        Fields::Unnamed(iter.into_iter().collect())
    }
}

/// A dynamic record, written with `{ ... }`.
///
/// Note that this library does not actually enforce that map keys and
/// values are homogeneous, but this is still required for well-formed RON.
///
/// Corresponds to the `serde` type `map`.
#[derive(Clone)]
pub struct Map<Key: Eq + Hash, Val> {
    raw: Box<indexmap::IndexMap<Key, Val>>,
}

impl<Key: Eq + Hash, Val> Default for Map<Key, Val> {
    fn default() -> Self {
        Map {
            raw: Default::default(),
        }
    }
}

impl<Key: Eq + Hash, Val> Map<Key, Val> {
    pub fn new() -> Self {
        Map::default()
    }
    pub fn iter(&self) -> impl Iterator<Item = (&Key, &Val)> {
        self.raw.iter()
    }
    pub fn len(&self) -> usize {
        self.raw.len()
    }
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }
}

impl<Key: Eq + Hash, Val> FromIterator<(Key, Val)> for Map<Key, Val> {
    fn from_iter<T: IntoIterator<Item = (Key, Val)>>(iter: T) -> Self {
        Map {
            raw: Box::new(iter.into_iter().collect()),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Sign {
    Positive,
    Negative,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Integer {
    pub(crate) raw: u128,
}

#[derive(Copy, Clone)]
pub struct Float {
    pub(crate) raw: f64,
}

// indexmap::IndexMap doesn't provide Hash
// also manually implement Partial/Eq to match Hash
impl<Key: Eq + Hash, Val: Eq> Eq for Map<Key, Val> {}
impl<Key: Eq + Hash, Val: PartialEq> PartialEq for Map<Key, Val> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}
impl<Key: Eq + Hash, Val: Hash> Hash for Map<Key, Val> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.iter().for_each(|x| x.hash(state))
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
        fn from_opaque(i: Integer) -> Option<Self>;
    }

    pub trait float: Copy {
        fn make_opaque(f: Self) -> Float;
        fn from_opaque(f: Float) -> Self;
    }

    macro_rules! impl_uint {
        ($($t:ty)*) => {$(
            impl uint for $t {
                fn make_opaque(i: Self) -> Integer {
                    Integer { raw: i.into() }
                }
                fn from_opaque(i: Integer) -> Option<Self> {
                    i.raw.try_into().ok()
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
                fn from_opaque(f: Float) -> Self {
                    f.raw as _
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
    #[allow(non_camel_case_types)]
    pub fn as_int<uint: sealed::uint>(self) -> Option<uint> {
        uint::from_opaque(self)
    }
}

impl Float {
    #[allow(non_camel_case_types)]
    pub fn as_float<float: sealed::float>(self) -> float {
        float::from_opaque(self)
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
            Value::Signed(Sign::Positive, int) | Value::Unsigned(int) => int.as_int(),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::Signed(Sign::Positive, int) | Value::Unsigned(int) => {
                let u: u64 = int.as_int()?;
                u.try_into().ok()
            }
            Value::Signed(Sign::Negative, int) => match int.as_int()? {
                u @ 0..=I64_MAX_AS_U64 => Some(-(u as i64)),
                u @ I64_MIN_AS_U64 => Some(u as i64),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn as_u128(&self) -> Option<u128> {
        match *self {
            Value::Signed(Sign::Positive, int) | Value::Unsigned(int) => int.as_int(),
            _ => None,
        }
    }

    pub fn as_i128(&self) -> Option<i128> {
        match *self {
            Value::Signed(Sign::Positive, int) | Value::Unsigned(int) => {
                let u: u128 = int.as_int()?;
                u.try_into().ok()
            }
            Value::Signed(Sign::Negative, int) => match int.as_int()? {
                u @ 0..=I128_MAX_AS_U128 => Some(-(u as i128)),
                u @ I128_MIN_AS_U128 => Some(u as i128),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        match *self {
            Value::Float(this) => Some(this.as_float()),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::Float(this) => Some(this.as_float()),
            _ => None,
        }
    }
}

const I64_MAX_AS_U64: u64 = i64::MAX as u64;
const I64_MIN_AS_U64: u64 = i64::MIN as u64;
const I128_MAX_AS_U128: u128 = i128::MAX as u128;
const I128_MIN_AS_U128: u128 = i128::MIN as u128;

pub fn to_value<T>(value: T) -> crate::Result<Value>
where
    T: serde::Serialize,
{
    value.serialize(transcode_ser::Serializer)
}

pub fn from_value<T>(value: Value) -> crate::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    T::deserialize(value)
}
