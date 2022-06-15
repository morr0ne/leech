use serde::{de, ser};
use std::{fmt::Display, str::Utf8Error};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Serde(String),
    #[error("Unexpected end of file")]
    Eof,
    #[error("Trailing bytes")]
    TrailingBytes,
    #[error("Syntax error")]
    Syntax,
    #[error("Leading zeros are forbiden")]
    LeadingZero,
    #[error("Negative zero is not a valid bencode number")]
    NegativeZero,
    #[error("")]
    InvalidType,
    #[error("")]
    UnexpectedToken,
    #[error("")]
    Utf8(#[from] Utf8Error),
    #[error("")]
    Io(#[from] std::io::Error),
    #[error("Unsupported type \"{0}\"")]
    Unsupported(&'static str),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Serde(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Serde(msg.to_string())
    }
}
