use {
    super::*,
    serde::de::{self, *},
    std::fmt,
};

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Value;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("any value")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Bool(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Signed(
                    if v >= 0 {
                        Sign::Positive
                    } else {
                        Sign::Negative
                    },
                    (v.abs() as u64).into(),
                ))
            }

            fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Signed(
                    if v >= 0 {
                        Sign::Positive
                    } else {
                        Sign::Negative
                    },
                    (v.abs() as u128).into(),
                ))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Unsigned(v.into()))
            }

            fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Unsigned(v.into()))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Float(v.into()))
            }

            fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Char(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_string(v.into())
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::String(v.into_boxed_str()))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_byte_buf(v.to_vec())
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Bytes(v))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Struct(Struct {
                    name: Some("None"),
                    fields: None,
                }))
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Ok(Value::Struct(Struct {
                    name: Some("Some"),
                    fields: Some(Box::new(Fields::Unnamed(vec![
                        deserializer.deserialize_any(self)?
                    ]))),
                }))
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Struct(Struct {
                    name: None,
                    fields: None,
                }))
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Ok(Value::Struct(Struct {
                    name: None, // unknown
                    fields: Some(Box::new(Fields::Unnamed(vec![
                        deserializer.deserialize_any(self)?
                    ]))),
                }))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut arr = seq
                    .size_hint()
                    .map(Vec::with_capacity)
                    .unwrap_or_else(Vec::new);
                while let Some(el) = seq.next_element()? {
                    arr.push(el);
                }
                Ok(Value::Array(arr))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut m = map
                    .size_hint()
                    .map(indexmap::IndexMap::with_capacity)
                    .unwrap_or_else(indexmap::IndexMap::new);
                while let Some((key, value)) = map.next_entry()? {
                    m.insert(key, value);
                }
                Ok(Value::Map(Map { raw: Box::new(m) }))
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let _ = data;
                Err(de::Error::custom("cannot deserialize_any an enum"))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}
