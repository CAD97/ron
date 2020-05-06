use {
    super::*,
    serde::ser::{self, *},
};

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Struct(value) => value.serialize(serializer),
            Value::Map(value) => value.serialize(serializer),
            Value::Array(value) => value.serialize(serializer),
            Value::String(value) => value.serialize(serializer),
            Value::Bytes(value) => value.serialize(serializer),
            Value::Bool(value) => value.serialize(serializer),
            Value::Signed(..) => {
                if let Some(value) = self.as_i64() {
                    value.serialize(serializer)
                } else if let Some(value) = self.as_i128() {
                    value.serialize(serializer)
                } else {
                    Err(ser::Error::custom("signed integer outside i128 range"))
                }
            }
            Value::Unsigned(..) => {
                if let Some(value) = self.as_u64() {
                    value.serialize(serializer)
                } else if let Some(value) = self.as_u128() {
                    value.serialize(serializer)
                } else {
                    Err(ser::Error::custom("unsigned integer outside u128 range"))
                }
            }
            Value::Float(..) => {
                if let Some(value) = self.as_f64() {
                    value.serialize(serializer)
                } else {
                    unreachable!()
                }
            }
            Value::Char(value) => value.serialize(serializer),
        }
    }
}

impl Serialize for Struct {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match (self.name, self.fields.as_deref()) {
            (Some(name), Some(Fields::Named(fields))) => {
                let mut s = serializer.serialize_struct(name, fields.len())?;
                for (name, field) in fields.iter() {
                    s.serialize_field(name, field)?;
                }
                s.end()
            }
            (Some(name), Some(Fields::Unnamed(fields))) => {
                let mut s = serializer.serialize_tuple_struct(name, fields.len())?;
                for field in fields.iter() {
                    s.serialize_field(field)?;
                }
                s.end()
            }
            (Some(name), None) => serializer.serialize_unit_struct(name),
            (None, Some(Fields::Named(fields))) => {
                // NB: this is not technically a valid syn structure, but is valid RON
                // So use the closest equivalent: a named struct with a null name
                let mut s = serializer.serialize_struct("", fields.len())?;
                for (name, field) in fields.iter() {
                    s.serialize_field(name, field)?;
                }
                s.end()
            }
            (None, Some(Fields::Unnamed(fields))) => {
                let mut s = serializer.serialize_tuple(fields.len())?;
                for field in fields.iter() {
                    s.serialize_element(field)?;
                }
                s.end()
            }
            (None, None) => serializer.serialize_unit(),
        }
    }
}

impl<K: Eq + Hash, V> Serialize for Map<K, V>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_map(Some(self.len()))?;
        for (key, value) in self.iter() {
            s.serialize_entry(key, value)?;
        }
        s.end()
    }
}
