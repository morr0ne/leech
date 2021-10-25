use crate::token::Token;
use std::fmt::Display;

use super::to_bencode::ToBencode;

pub trait PrintableInteger: Display {}

macro_rules! impl_integer {
    ($($type:ty)*) => {$(
        impl PrintableInteger for $type {}
    )*}
}

impl_integer!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);

pub struct Encoder {
    pub(crate) bytes: Vec<u8>,
}
pub struct DictionaryEncoder<'a>(&'a mut Encoder);

impl Encoder {
    pub fn new() -> Encoder {
        Encoder { bytes: Vec::new() }
    }

    pub(crate) fn emit_token(&mut self, token: Token) {
        match token {
            Token::ByteString(byte_string) => {
                // Writing to a vec can't fail
                let length = byte_string.len().to_string();
                self.bytes.extend_from_slice(length.as_bytes());
                self.bytes.push(b':');
                self.bytes.extend_from_slice(byte_string);
            }
            Token::Integer(integer) => {
                // Alas, this doesn't verify that the given number is valid
                self.bytes.push(b'i');
                self.bytes.extend_from_slice(integer);
                self.bytes.push(b'e');
            }
            Token::ListStart => self.bytes.push(b'l'),
            Token::DictionaryStart => self.bytes.push(b'd'),
            Token::End => self.bytes.push(b'e'),
        }
    }

    /// Emit an arbitrary encodable object
    pub fn emit<E: ToBencode>(&mut self, value: E) {
        value.encode(self)
    }

    /// Emit a string
    pub fn emit_byte_string(&mut self, value: &str) {
        self.emit_token(Token::ByteString(value.as_bytes()))
    }

    /// Emit a byte array
    pub fn emit_byte_array(&mut self, value: &[u8]) {
        self.emit_token(Token::ByteString(value))
    }

    /// Emit an integer
    pub fn emit_integer<T: PrintableInteger>(&mut self, integer: T) {
        self.emit_token(Token::Integer(integer.to_string().as_bytes()))
    }

    pub fn emit_list<F>(&mut self, list_callback: F)
    where
        F: FnOnce(&mut Encoder),
    {
        self.emit_token(Token::ListStart);
        list_callback(self);
        self.emit_token(Token::End)
    }

    pub fn emit_dictionary<F>(&mut self, content_callback: F)
    where
        F: FnOnce(DictionaryEncoder),
    {
        self.emit_token(Token::DictionaryStart);
        content_callback(DictionaryEncoder(self));
        self.emit_token(Token::End)
    }
}

impl Default for Encoder {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> DictionaryEncoder<'a> {
    /// Emit a key/value pair
    pub fn emit_pair<E>(&mut self, key: &[u8], value: E)
    where
        E: ToBencode,
    {
        self.0.emit_token(Token::ByteString(key));
        value.encode(self.0);
    }
}
