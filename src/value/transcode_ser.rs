use {
    super::*,
    crate::error::{Error, Result},
    serde::ser::{self, *},
};

pub struct Serializer;

impl ser::Serializer for Serializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = SeqSerializer;
    type SerializeTuple = SeqSerializer;
    type SerializeTupleStruct = SeqSerializer;
    type SerializeTupleVariant = SeqSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = RecSerializer;
    type SerializeStructVariant = RecSerializer;

    fn serialize_bool(self, v: bool) -> Result<Value> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Value> {
        self.serialize_i64(v.into())
    }

    fn serialize_i16(self, v: i16) -> Result<Value> {
        self.serialize_i64(v.into())
    }

    fn serialize_i32(self, v: i32) -> Result<Value> {
        self.serialize_i64(v.into())
    }

    fn serialize_i64(self, v: i64) -> Result<Value> {
        Ok(Value::Signed(
            if v >= 0 {
                Sign::Positive
            } else {
                Sign::Negative
            },
            (v.abs() as u64).into(),
        ))
    }

    fn serialize_i128(self, v: i128) -> Result<Value> {
        Ok(Value::Signed(
            if v >= 0 {
                Sign::Positive
            } else {
                Sign::Negative
            },
            (v.abs() as u128).into(),
        ))
    }

    fn serialize_u8(self, v: u8) -> Result<Value> {
        self.serialize_u64(v.into())
    }

    fn serialize_u16(self, v: u16) -> Result<Value> {
        self.serialize_u64(v.into())
    }

    fn serialize_u32(self, v: u32) -> Result<Value> {
        self.serialize_u64(v.into())
    }

    fn serialize_u64(self, v: u64) -> Result<Value> {
        Ok(Value::Unsigned(v.into()))
    }

    fn serialize_u128(self, v: u128) -> Result<Value> {
        Ok(Value::Unsigned(v.into()))
    }

    fn serialize_f32(self, v: f32) -> Result<Value> {
        self.serialize_f64(v.into())
    }

    fn serialize_f64(self, v: f64) -> Result<Value> {
        Ok(Value::Float(v.into()))
    }

    fn serialize_char(self, v: char) -> Result<Value> {
        Ok(Value::Char(v))
    }

    fn serialize_str(self, v: &str) -> Result<Value> {
        Ok(Value::String(v.into()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Value> {
        Ok(Value::Bytes(v.into()))
    }

    fn serialize_none(self) -> Result<Value> {
        self.serialize_unit_variant("Option", 0, "None")
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Value>
    where
        T: Serialize,
    {
        self.serialize_newtype_variant("Option", 1, "Some", value)
    }

    fn serialize_unit(self) -> Result<Value> {
        Ok(Value::Struct(Struct {
            name: None,
            fields: None,
        }))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Value> {
        Ok(Value::Struct(Struct {
            name: Some(name),
            fields: None,
        }))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Value> {
        let _ = name;
        let _ = variant_index;
        self.serialize_unit_struct(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result<Value>
    where
        T: Serialize,
    {
        let _ = name;
        // NB: Serializers are encouraged to treat newtype structs
        // as insignificant wrappers around the data they contain.
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value>
    where
        T: Serialize,
    {
        let _ = name;
        let _ = variant_index;
        let mut tuple_struct = self.serialize_tuple_struct(variant, 1)?;
        SerializeTupleStruct::serialize_field(&mut tuple_struct, value)?;
        SerializeTupleStruct::end(tuple_struct)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SeqSerializer {
            name: None,
            buf: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        Ok(SeqSerializer {
            name: None,
            buf: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(SeqSerializer {
            name: Some(name),
            buf: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let _ = name;
        let _ = variant_index;
        Ok(SeqSerializer {
            name: Some(variant),
            buf: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(MapSerializer {
            buf: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(RecSerializer {
            name,
            buf: Vec::with_capacity(len),
        })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let _ = name;
        let _ = variant_index;
        Ok(RecSerializer {
            name: variant,
            buf: Vec::with_capacity(len),
        })
    }
}

pub struct SeqSerializer {
    name: Option<&'static str>,
    buf: Vec<Value>,
}

impl ser::SerializeSeq for SeqSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.buf.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        assert_eq!(self.name, None);
        Ok(Value::Array(self.buf))
    }
}

impl ser::SerializeTuple for SeqSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.buf.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Struct(Struct {
            name: self.name,
            fields: Some(Box::new(self.buf.into_iter().collect())),
        }))
    }
}

impl ser::SerializeTupleStruct for SeqSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.buf.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Struct(Struct {
            name: self.name,
            fields: Some(Box::new(self.buf.into_iter().collect())),
        }))
    }
}

impl ser::SerializeTupleVariant for SeqSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.buf.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Struct(Struct {
            name: self.name,
            fields: Some(Box::new(self.buf.into_iter().collect())),
        }))
    }
}

pub struct MapSerializer {
    buf: Vec<(Value, Option<Value>)>,
}

impl ser::SerializeMap for MapSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.buf.push((to_value(key)?, None));
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        assert!(self.buf.last_mut().unwrap().1.is_none());
        self.buf.last_mut().unwrap().1 = Some(to_value(value)?);
        Ok(())
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: Serialize,
        V: Serialize,
    {
        self.buf.push((to_value(key)?, Some(to_value(value)?)));
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Map(
            self.buf
                .into_iter()
                .map(|(key, val)| (key, val.unwrap()))
                .collect(),
        ))
    }
}

pub struct RecSerializer {
    name: &'static str,
    buf: Vec<(&'static str, Value)>,
}

impl ser::SerializeStruct for RecSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.buf.push((key, to_value(value)?));
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Struct(Struct {
            name: Some(self.name),
            fields: Some(Box::new(self.buf.into_iter().collect())),
        }))
    }
}

impl ser::SerializeStructVariant for RecSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.buf.push((key, to_value(value)?));
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Struct(Struct {
            name: Some(self.name),
            fields: Some(Box::new(self.buf.into_iter().collect())),
        }))
    }
}
