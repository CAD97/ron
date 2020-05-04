use {
    crate::parse::Position,
    serde::{de, ser},
    std::{
        fmt::{self, Display},
        io,
    },
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct Error {
    pos: Position,
    code: Code,
}

#[derive(Debug)]
enum Code {
    Message(Box<str>),
    Io(io::Error),
    Base64(base64::DecodeError),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::custom(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::custom(msg.to_string())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn 'static + std::error::Error)> {
        match &self.code {
            Code::Io(err) => Some(err),
            Code::Base64(err) => Some(err),
            _ => None,
        }
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.pos != Position::default() {
            write!(f, "{}:{}: {}", self.pos.line, self.pos.col, self.code)
        } else {
            write!(f, "{}", self.code)
        }
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Code::Message(msg) => f.write_str(msg),
            Code::Io(err) => write!(f, "{}", err),
            Code::Base64(err) => write!(f, "{}", err),
        }
    }
}

impl Error {
    /// Recover position information from string.
    /// erased-serde round-trips errors through Error::custom.
    fn custom(mut msg: String) -> Error {
        if let Some(colon1) = msg.find(':') {
            if let Some(colon2) = msg[colon1 + 1..].find(':') {
                if let Ok(line) = msg[..colon1].parse() {
                    if let Ok(col) = msg[colon1 + 1..colon2].parse() {
                        msg.replace_range(..=colon2, "");
                        let pos = Position { line, col };
                        let code = Code::Message(msg.into_boxed_str());
                        return Error { pos, code };
                    }
                }
            }
        }
        Error {
            pos: Position::default(),
            code: Code::Message(msg.into_boxed_str()),
        }
    }
}
