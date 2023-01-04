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
    #[error("Invalid type")]
    InvalidType,
    #[error("Unexpected Token {found} at index {index}, expected {expected}")]
    UnexpectedToken {
        expected: &'static str,
        found: u8,
        index: usize,
    },
    #[error("Error while parsing utf8 value")]
    Utf8(#[from] Utf8Error),
    #[error("Io Error")]
    Io(#[from] std::io::Error),
    #[error("Unsupported type \"{0}\"")]
    Unsupported(&'static str),
    /// Unexpected end of file while parsing a byte string.
    ///
    /// This usually happens when the specified length is incorrect.
    #[error("Unexpected end of file while parsing a byte string")]
    EofWhileParsingByteString,

    #[error("Expected a dictionary, found a byte string instead")]
    ExpectedDictionaryFoundByteString,
    #[error("Expected a dictionary, found an integer instead")]
    ExpectedDictionaryFoundInteger,
    #[error("Expected a dictionary, found a list instead")]
    ExpectedDictionaryFoundList,

    #[error("Unsorted keys")]
    UnsortedKeys,

    #[error("Integer out of bound")]
    OutOfBound,
}

impl Error {
    pub const fn unexpected_token(expected: &'static str, found: u8, index: usize) -> Self {
        Self::UnexpectedToken {
            expected,
            found,
            index,
        }
    }
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
