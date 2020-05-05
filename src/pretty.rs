use {
    crate::{
        value::{Float, Integer, Sign},
    },
    serde::{Deserialize, Serialize},
    std::{fmt, io, slice},
};

type Result = io::Result<()>;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Indent up to this level of nesting.
    /// Use a value of 0 for no indentation.
    #[serde(default = "usize::max_value")]
    pub depth_limit: usize,
    #[serde(default = "Indent::default")]
    /// The indentation style to use.
    pub indent: Indent,
    /// This is used to allow record update syntax (`{ ..default() }`),
    /// which `#[non_exhaustive]` or private members do not at this time.
    #[doc(hidden)]
    #[serde(skip)]
    pub __non_exhaustive: (),
}

pub struct Formatter {
    config: Config,
    depth: usize,
    not_first: bool,
    buf: String,
}

const INITIAL_STRING_LENGTH: usize = 256;

impl Default for Formatter {
    fn default() -> Self {
        Formatter {
            config: Default::default(),
            depth: 0,
            not_first: false,
            // because of lexical, make sure string is initialized to start
            buf: unsafe { String::from_utf8_unchecked(vec![0; INITIAL_STRING_LENGTH]) },
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            depth_limit: usize::MAX,
            indent: Indent::default(),
            __non_exhaustive: (),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Indent {
    Spaces(u8),
    Tabs(u8),
}

impl Default for Indent {
    fn default() -> Self {
        Indent::Spaces(4)
    }
}

impl fmt::Display for Indent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // forgive me
        #[repr(C)]
        union ConstTransmute<T: Copy, U: Copy> {
            from: T,
            to: U,
        }
        const SPACES: &str = unsafe { ConstTransmute::<&[u8], &str> { from: &[b' '; 255] }.to };
        const TABS: &str = unsafe {
            ConstTransmute::<&[u8], &str> {
                from: &[b'\t'; 255],
            }
            .to
        };

        match *self {
            Indent::Spaces(count) => f.write_str(&SPACES[..count as usize]),
            Indent::Tabs(count) => f.write_str(&TABS[..count as usize]),
        }
    }
}

impl Config {
    fn write_space<W: io::Write>(&self, w: &mut W) -> Result {
        if self.depth_limit > 0 {
            write!(w, " ")
        } else {
            Ok(())
        }
    }

    fn write_indent<W: io::Write>(&self, w: &mut W, depth: usize) -> Result {
        if depth > self.depth_limit {
            self.write_space(w)
        } else {
            writeln!(w)?;
            for _ in 0..depth {
                write!(w, "{}", self.indent)?;
            }
            Ok(())
        }
    }
}

impl Formatter {
    /// lexical does not provide a growable-buffer taking form,
    /// so grab a raw buffer that should be large enough.
    /// This is why we zero-initialize the string buffer.
    fn raw_buf(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.buf.as_mut_ptr(), INITIAL_STRING_LENGTH) }
    }

    pub(crate) fn write_string<W: io::Write>(&mut self, w: &mut W, v: &str) -> Result {
        write!(w, r#""{}""#, v.escape_debug())
    }

    pub(crate) fn write_bytes<W: io::Write>(&mut self, w: &mut W, v: &[u8]) -> Result {
        self.buf.clear();
        base64::encode_config_buf(v, base64::STANDARD, &mut self.buf);
        write!(w, r#"b"{}""#, &self.buf)
    }

    pub(crate) fn write_bool<W: io::Write>(&mut self, w: &mut W, v: bool) -> Result {
        write!(w, "{}", v)
    }

    pub(crate) fn write_signed<W: io::Write>(
        &mut self,
        w: &mut W,
        (sign, v): (Sign, Integer),
    ) -> Result {
        match sign {
            Sign::Positive => write!(w, "+")?,
            Sign::Negative => write!(w, "-")?,
        }
        self.write_unsigned(w, v)
    }

    pub(crate) fn write_unsigned<W: io::Write>(&mut self, w: &mut W, v: Integer) -> Result {
        self.buf.clear();
        w.write_all(lexical::ToLexical::to_lexical(v.raw, self.raw_buf()))
    }

    pub(crate) fn write_float<W: io::Write>(&mut self, w: &mut W, v: Float) -> Result {
        self.buf.clear();
        w.write_all(lexical::ToLexical::to_lexical(v.raw, self.raw_buf()))
    }

    pub(crate) fn write_char<W: io::Write>(&mut self, w: &mut W, v: char) -> Result {
        write!(w, "'{}'", v.escape_debug())
    }

    pub(crate) fn write_unit<W: io::Write>(&mut self, w: &mut W, v: Option<&str>) -> Result {
        if let Some(v) = v {
            write!(w, "{}", v)
        } else {
            write!(w, "()")
        }
    }

    pub(crate) fn begin_struct<W: io::Write>(&mut self, w: &mut W, v: Option<&str>) -> Result {
        self.not_first = false;
        self.depth += 1;
        if let Some(v) = v {
            write!(w, "{}", v)?;
        }
        write!(w, "(")
    }

    pub(crate) fn begin_struct_field<W: io::Write>(
        &mut self,
        w: &mut W,
        v: Option<&str>,
    ) -> Result {
        if self.not_first {
            write!(w, ",")?;
        }
        self.config.write_indent(w, self.depth)?;
        if let Some(v) = v {
            write!(w, "{}:", v)?;
            self.config.write_space(w)?;
        }
        Ok(())
    }

    pub(crate) fn end_struct_field<W: io::Write>(&mut self, w: &mut W) -> Result {
        let _ = w;
        self.not_first = true;
        Ok(())
    }

    pub(crate) fn end_struct<W: io::Write>(&mut self, w: &mut W) -> Result {
        self.depth -= 1;
        if self.not_first {
            self.config.write_indent(w, self.depth)?;
        }
        self.not_first = true;
        write!(w, ")")
    }

    pub(crate) fn begin_map<W: io::Write>(&mut self, w: &mut W) -> Result {
        self.not_first = false;
        self.depth += 1;
        write!(w, "{{")
    }

    pub(crate) fn begin_map_key<W: io::Write>(&mut self, w: &mut W) -> Result {
        if self.not_first {
            write!(w, ",")?;
        }
        self.config.write_indent(w, self.depth)
    }

    pub(crate) fn end_map_key<W: io::Write>(&mut self, w: &mut W) -> Result {
        let _ = w;
        self.not_first = true;
        Ok(())
    }

    pub(crate) fn begin_map_value<W: io::Write>(&mut self, w: &mut W) -> Result {
        write!(w, ":")?;
        self.config.write_space(w)
    }

    pub(crate) fn end_map_value<W: io::Write>(&mut self, w: &mut W) -> Result {
        let _ = w;
        self.not_first = true;
        Ok(())
    }

    pub(crate) fn end_map<W: io::Write>(&mut self, w: &mut W) -> Result {
        self.depth -= 1;
        if self.not_first {
            self.config.write_indent(w, self.depth)?;
        }
        self.not_first = true;
        write!(w, "}}")
    }

    pub(crate) fn begin_array<W: io::Write>(&mut self, w: &mut W) -> Result {
        self.not_first = false;
        self.depth += 1;
        write!(w, "[")
    }

    pub(crate) fn begin_array_member<W: io::Write>(&mut self, w: &mut W) -> Result {
        if self.not_first {
            write!(w, ",")?;
        }
        self.config.write_indent(w, self.depth)
    }

    pub(crate) fn end_array_member<W: io::Write>(&mut self, w: &mut W) -> Result {
        let _ = w;
        self.not_first = true;
        Ok(())
    }

    pub(crate) fn end_array<W: io::Write>(&mut self, w: &mut W) -> Result {
        self.depth -= 1;
        if self.not_first {
            self.config.write_indent(w, self.depth)?;
        }
        self.not_first = true;
        write!(w, "]")
    }
}
