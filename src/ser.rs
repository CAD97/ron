use {
    crate::{error::Error, pretty::Formatter, value::Sign},
    serde::ser::{self, *},
    std::io,
};

type Result<T = ()> = crate::Result<T>;

pub struct Serializer<W> {
    writer: W,
    formatter: Formatter,
}

impl<'a, W> ser::Serializer for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = not_public_api::Compound<'a, W>;
    type SerializeTuple = not_public_api::Compound<'a, W>;
    type SerializeTupleStruct = not_public_api::Compound<'a, W>;
    type SerializeTupleVariant = not_public_api::Compound<'a, W>;
    type SerializeMap = not_public_api::Compound<'a, W>;
    type SerializeStruct = not_public_api::Compound<'a, W>;
    type SerializeStructVariant = not_public_api::Compound<'a, W>;

    fn serialize_bool(self, v: bool) -> Result {
        self.formatter.write_bool(&mut self.writer, v)?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result {
        self.serialize_i128(v.into())
    }

    fn serialize_i16(self, v: i16) -> Result {
        self.serialize_i128(v.into())
    }

    fn serialize_i32(self, v: i32) -> Result {
        self.serialize_i128(v.into())
    }

    fn serialize_i64(self, v: i64) -> Result {
        self.serialize_i128(v.into())
    }

    fn serialize_i128(self, v: i128) -> Result {
        let sign = if v >= 0 {
            Sign::Positive
        } else {
            Sign::Negative
        };
        self.formatter
            .write_signed(&mut self.writer, (sign, (v.abs() as u128).into()))?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result {
        self.serialize_u128(v.into())
    }

    fn serialize_u16(self, v: u16) -> Result {
        self.serialize_u128(v.into())
    }

    fn serialize_u32(self, v: u32) -> Result {
        self.serialize_u128(v.into())
    }

    fn serialize_u64(self, v: u64) -> Result {
        self.serialize_u128(v.into())
    }

    fn serialize_u128(self, v: u128) -> Result {
        self.formatter.write_unsigned(&mut self.writer, v.into())?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result {
        self.serialize_f64(v.into())
    }

    fn serialize_f64(self, v: f64) -> Result {
        self.formatter.write_float(&mut self.writer, v.into())?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result {
        self.formatter.write_char(&mut self.writer, v)?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result {
        self.formatter.write_string(&mut self.writer, v)?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result {
        self.formatter.write_bytes(&mut self.writer, v)?;
        Ok(())
    }

    fn serialize_none(self) -> Result {
        self.serialize_unit_variant("Option", 0, "None")
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result
    where
        T: Serialize,
    {
        self.serialize_newtype_variant("Option", 1, "Some", value)
    }

    fn serialize_unit(self) -> Result {
        self.formatter.write_unit(&mut self.writer, None)?;
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result {
        self.formatter.write_unit(&mut self.writer, Some(name))?;
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result {
        let _ = name;
        let _ = variant_index;
        self.serialize_unit_struct(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result
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
    ) -> Result
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
        let _ = len;
        self.formatter.begin_array(&mut self.writer)?;
        Ok(not_public_api::Compound { ser: self })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        let _ = len;
        self.formatter.begin_struct(&mut self.writer, None)?;
        Ok(not_public_api::Compound { ser: self })
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        let _ = len;
        self.formatter.begin_struct(&mut self.writer, Some(name))?;
        Ok(not_public_api::Compound { ser: self })
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        let _ = name;
        let _ = variant_index;
        self.serialize_tuple_struct(variant, len)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        let _ = len;
        self.formatter.begin_map(&mut self.writer)?;
        Ok(not_public_api::Compound { ser: self })
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        let _ = len;
        self.formatter.begin_struct(&mut self.writer, Some(name))?;
        Ok(not_public_api::Compound { ser: self })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        let _ = name;
        let _ = variant_index;
        self.serialize_struct(variant, len)
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

mod not_public_api {
    use super::*;

    pub struct Compound<'a, W> {
        pub(super) ser: &'a mut Serializer<W>,
    }

    impl<W> ser::SerializeSeq for Compound<'_, W>
    where
        W: io::Write,
    {
        type Ok = ();
        type Error = Error;

        fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result
        where
            T: Serialize,
        {
            self.ser
                .formatter
                .begin_array_member(&mut self.ser.writer)?;
            value.serialize(&mut *self.ser)?;
            self.ser.formatter.end_array_member(&mut self.ser.writer)?;
            Ok(())
        }

        fn end(self) -> Result {
            self.ser.formatter.end_array(&mut self.ser.writer)?;
            Ok(())
        }
    }

    impl<W> SerializeTuple for Compound<'_, W>
    where
        W: io::Write,
    {
        type Ok = ();
        type Error = Error;

        fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result
        where
            T: Serialize,
        {
            self.ser
                .formatter
                .begin_struct_field(&mut self.ser.writer, None)?;
            value.serialize(&mut *self.ser)?;
            self.ser.formatter.end_struct_field(&mut self.ser.writer)?;
            Ok(())
        }

        fn end(self) -> Result {
            self.ser.formatter.end_struct(&mut self.ser.writer)?;
            Ok(())
        }
    }

    impl<W> SerializeTupleStruct for Compound<'_, W>
    where
        W: io::Write,
    {
        type Ok = ();
        type Error = Error;

        fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result
        where
            T: Serialize,
        {
            SerializeTuple::serialize_element(self, value)
        }

        fn end(self) -> Result {
            SerializeTuple::end(self)
        }
    }

    impl<W> SerializeTupleVariant for Compound<'_, W>
    where
        W: io::Write,
    {
        type Ok = ();
        type Error = Error;

        fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result
        where
            T: Serialize,
        {
            SerializeTuple::serialize_element(self, value)
        }

        fn end(self) -> Result {
            SerializeTuple::end(self)
        }
    }

    impl<W> SerializeMap for Compound<'_, W>
    where
        W: io::Write,
    {
        type Ok = ();
        type Error = Error;

        fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result
        where
            T: Serialize,
        {
            self.ser.formatter.begin_map_key(&mut self.ser.writer)?;
            key.serialize(&mut *self.ser)?;
            self.ser.formatter.end_map_key(&mut self.ser.writer)?;
            Ok(())
        }

        fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result
        where
            T: Serialize,
        {
            self.ser.formatter.begin_map_value(&mut self.ser.writer)?;
            value.serialize(&mut *self.ser)?;
            self.ser.formatter.end_map_value(&mut self.ser.writer)?;
            Ok(())
        }

        fn end(self) -> Result {
            self.ser.formatter.end_map(&mut self.ser.writer)?;
            Ok(())
        }
    }

    impl<W> SerializeStruct for Compound<'_, W>
    where
        W: io::Write,
    {
        type Ok = ();
        type Error = Error;

        fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result
        where
            T: Serialize,
        {
            self.ser
                .formatter
                .begin_struct_field(&mut self.ser.writer, Some(key))?;
            value.serialize(&mut *self.ser)?;
            self.ser.formatter.end_struct_field(&mut self.ser.writer)?;
            Ok(())
        }

        fn end(self) -> Result {
            self.ser.formatter.end_struct(&mut self.ser.writer)?;
            Ok(())
        }
    }

    impl<W> SerializeStructVariant for Compound<'_, W>
    where
        W: io::Write,
    {
        type Ok = ();
        type Error = Error;

        fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result
        where
            T: Serialize,
        {
            SerializeStruct::serialize_field(self, key, value)
        }

        fn end(self) -> Result {
            SerializeStruct::end(self)
        }
    }
}

pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ?Sized + Serialize,
{
    value.serialize(&mut Serializer {
        writer,
        formatter: Formatter::default(),
    })
}

pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    let mut writer = Vec::new();
    to_writer(&mut writer, value)?;
    Ok(writer)
}

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let vec = to_vec(value)?;
    let string = unsafe {
        // SAFETY: we only emit valid UTF-8
        String::from_utf8_unchecked(vec)
    };
    Ok(string)
}
