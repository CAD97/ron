use {
    super::*,
    std::{
        fmt::{self, Debug},
        io::{self, prelude::*},
        str,
    },
};

struct IoFmtWrite<W>(W);

impl<W: fmt::Write> io::Write for IoFmtWrite<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let buf =
            str::from_utf8(buf).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        self.0
            .write_str(buf)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Struct(value) => value.fmt(f),
            Value::Map(value) => value.fmt(f),
            Value::Array(value) => value.fmt(f),
            Value::String(value) => value.fmt(f),
            Value::Bytes(value) => {
                f.write_str(r#"b""#)?;
                base64::write::EncoderWriter::new(&mut IoFmtWrite(&mut *f), base64::STANDARD)
                    .write_all(value)
                    .map_err(|_| fmt::Error::default())?;
                f.write_str(r#"""#)
            }
            Value::Bool(value) => value.fmt(f),
            Value::Signed(sign, value) => sign.fmt(f).and_then(|()| value.fmt(f)),
            Value::Unsigned(value) => value.fmt(f),
            Value::Float(value) => value.fmt(f),
            Value::Char(value) => value.fmt(f),
        }
    }
}

impl Debug for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.name, self.fields.as_deref()) {
            (name, Some(Fields::Named(fields))) => {
                let mut d = f.debug_struct(name.unwrap_or("_"));
                for (name, field) in fields.iter() {
                    d.field(name, field);
                }
                d.finish()
            }
            (name, Some(Fields::Unnamed(fields))) => {
                let mut d = f.debug_tuple(name.unwrap_or(""));
                for field in fields.iter() {
                    d.field(field);
                }
                d.finish()
            }
            (Some(name), None) => f.write_str(name),
            (None, None) => ().fmt(f),
        }
    }
}

impl<K: Eq + Hash, V> Debug for Map<K, V>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_map();
        for (key, val) in self.iter() {
            d.entry(key, val);
        }
        d.finish()
    }
}

impl Debug for Sign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sign::Positive => f.write_str("+"),
            Sign::Negative => f.write_str("-"),
        }
    }
}

impl Debug for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.fmt(f)
    }
}

impl Debug for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.fmt(f)
    }
}
