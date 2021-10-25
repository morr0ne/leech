use std::collections::{LinkedList, VecDeque};

use crate::AsString;

use super::{Encoder, EncodingError};

pub trait ToBencode {
    fn to_bencode(&self) -> Result<Vec<u8>, EncodingError>
    where
        Self: Sized,
    {
        let mut encoder = Encoder::new();

        self.encode(&mut encoder);

        Ok(encoder.bytes)
    }

    fn encode(&self, encoder: &mut Encoder);
}

// Forwarding impls
impl<'a, E: 'a + ToBencode + Sized> ToBencode for &'a E {
    fn encode(&self, encoder: &mut Encoder) {
        E::encode(self, encoder)
    }
}

// Base type impls
impl<'a> ToBencode for &'a str {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.emit_byte_string(self)
    }
}

impl ToBencode for String {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.emit_byte_string(self)
    }
}

macro_rules! impl_encodable_integer {
    ($($type:ty)*) => {$(
        impl ToBencode for $type {
            fn encode(&self, encoder: &mut Encoder) {
                encoder.emit_integer(*self)
            }
        }
    )*}
}

impl_encodable_integer!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);

macro_rules! impl_encodable_iterable {
    ($($type:ident)*) => {$(
        impl <ContentT> ToBencode for $type<ContentT>
        where
            ContentT: ToBencode
        {
            fn encode(&self, encoder: &mut Encoder){
                encoder.emit_list(|e| {
                    for item in self {
                        e.emit(item);
                    }
                });
            }
        }
    )*}
}

impl_encodable_iterable!(Vec VecDeque LinkedList);

impl<I> ToBencode for AsString<I>
where
    I: AsRef<[u8]>,
{
    fn encode(&self, encoder: &mut Encoder) {
        encoder.emit_byte_array(self.0.as_ref());
    }
}

impl<I> AsRef<[u8]> for AsString<I>
where
    I: AsRef<[u8]>,
{
    fn as_ref(&self) -> &'_ [u8] {
        self.0.as_ref()
    }
}

impl<'a, I> From<&'a [u8]> for AsString<I>
where
    I: From<&'a [u8]>,
{
    fn from(content: &'a [u8]) -> Self {
        AsString(I::from(content))
    }
}

impl<'a, T> ToBencode for &'a [T]
where
    T: ToBencode,
{
    fn encode(&self, encoder: &mut Encoder) {
        encoder.emit_list(|e| {
            for item in *self {
                e.emit(item);
            }
        });
    }
}
