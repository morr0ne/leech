use std::{
    collections::HashMap,
    hash::{BuildHasher, Hash},
};

use super::{decoder::Decoder, object::Object};
use crate::error::{Error, Result};

///Basic trait for bencode based value deserialization.
pub trait FromBencode {
    /// Deserialize an object from its byte representation.
    fn from_bencode(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let mut decoder = Decoder::new(bytes);
        let object = decoder.next_object()?;

        object.map_or(
            // Err(Error::from(StructureError::UnexpectedEof)),
            Err(Error::Unknown),
            Self::bdecode,
        )
    }

    /// Deserialize an object from its intermediate bencode representation.
    fn bdecode(object: Object) -> Result<Self>
    where
        Self: Sized;
}

impl FromBencode for i64 {
    fn bdecode(object: Object) -> Result<Self>
    where
        Self: Sized,
    {
        let number = object.integer().unwrap();

        Ok(number)
    }
}

impl<T: FromBencode> FromBencode for Vec<T> {
    fn bdecode(object: Object) -> Result<Self>
    where
        Self: Sized,
    {
        let mut list = object.list().ok_or(Error::Unknown)?;
        let mut results = Vec::new();

        while let Some(object) = list.next_object()? {
            let item = T::bdecode(object)?;
            results.push(item);
        }

        Ok(results)
    }
}

impl FromBencode for String {
    fn bdecode(object: Object) -> Result<Self>
    where
        Self: Sized,
    {
        let content = object.byte_string().ok_or(Error::Unknown)?;
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
    fn bdecode(object: Object) -> Result<Self>
    where
        Self: Sized,
    {
        let mut dict = object.dictionary().ok_or(Error::Unknown)?;
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
    fn bdecode(object: Object) -> Result<Self> {
        object
            .byte_string()
            .map(Vec::from)
            .map(AsString)
            .ok_or(Error::Unknown)
    }
}
