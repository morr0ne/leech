use atoi::atoi;
use std::{
    collections::HashMap,
    hash::{BuildHasher, Hash},
};

use super::{decoder::Decoder, error::DecodingError, object::Object};

pub trait FromBencode {
    fn from_bencode(bytes: &[u8]) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut decoder = Decoder::new(bytes);
        let object = decoder.next_object()?;

        object.map_or(
            // Err(Error::from(StructureError::UnexpectedEof)),
            Err(DecodingError::Unknown),
            Self::bdecode,
        )
    }

    fn bdecode(object: Object) -> Result<Self, DecodingError>
    where
        Self: Sized;
}

macro_rules! impl_from_bencode_for_num {
    ($($type:ty)*) => {$(
        impl FromBencode for $type {

            fn bdecode(object: Object) -> Result<Self, DecodingError>
            where
                Self: Sized,
            {
                let number = object.integer().ok_or(DecodingError::Unknown)?;
                let number = atoi(number).ok_or(DecodingError::Unknown)?;

                Ok(number)
            }
        }
    )*}
}

impl_from_bencode_for_num!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);

impl<T: FromBencode> FromBencode for Vec<T> {
    fn bdecode(object: Object) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut list = object.list().ok_or(DecodingError::Unknown)?;
        let mut results = Vec::new();

        while let Some(object) = list.next_object()? {
            let item = T::bdecode(object)?;
            results.push(item);
        }

        Ok(results)
    }
}

impl FromBencode for String {
    fn bdecode(object: Object) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let content = object.byte_string().ok_or(DecodingError::Unknown)?;
        let content = String::from_utf8(content.to_vec()).unwrap(); // TODO: map proper error

        Ok(content)
    }
}

impl<K, V, H> FromBencode for HashMap<K, V, H>
where
    K: FromBencode + Hash + Eq,
    V: FromBencode,
    H: BuildHasher + Default,
{
    fn bdecode(object: Object) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut dict = object.dictionary().ok_or(DecodingError::Unknown)?;
        let mut result = HashMap::default();

        while let Some((key, value)) = dict.next_pair()? {
            let key = K::bdecode(Object::ByteString(key))?;
            let value = V::bdecode(value)?;

            result.insert(key, value);
        }

        Ok(result)
    }
}

/// Wrapper to allow `Vec<u8>` encoding as bencode string element.
#[derive(Debug)]
pub struct AsString(pub Vec<u8>);

impl FromBencode for AsString {
    fn bdecode(object: Object) -> Result<Self, DecodingError> {
        object
            .byte_string()
            .map(Vec::from)
            .map(AsString)
            .ok_or(DecodingError::Unknown)
    }
}
