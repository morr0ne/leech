
/*
The current code was adapted from the serde_bytes witch contains the following copyright notice.


Copyright (c) 2014 The Rust Project Developers

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.
*/


use std::{
    boxed::Box,
    cmp::{self, Ordering},
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    string::String,
    vec::Vec,
};

use serde::{
    de::{Deserialize, Deserializer, Error, SeqAccess, Visitor},
    ser::{Serialize, Serializer},
};

/// Wrapper around `Vec<u8>` to serialize and deserialize efficiently.
/// see <https://github.com/serde-rs/bytes>
#[derive(Clone, Default, Eq, Ord)]
pub struct ByteString {
    bytes: Vec<u8>,
}

impl ByteString {
    /// Construct a new, empty `ByteString`.
    pub fn new() -> Self {
        ByteString::from(Vec::new())
    }

    /// Construct a new, empty `ByteString` with the specified capacity.
    pub fn with_capacity(cap: usize) -> Self {
        ByteString::from(Vec::with_capacity(cap))
    }

    /// Wrap existing bytes in a `ByteString`.
    pub fn from<T: Into<Vec<u8>>>(bytes: T) -> Self {
        ByteString {
            bytes: bytes.into(),
        }
    }

    /// Unwrap the vector of byte underlying this `ByteString`.
    pub fn into_vec(self) -> Vec<u8> {
        self.bytes
    }

    // This would hit "cannot move out of borrowed content" if invoked through
    // the Deref impl; make it just work.
    #[doc(hidden)]
    pub fn into_boxed_slice(self) -> Box<[u8]> {
        self.bytes.into_boxed_slice()
    }

    #[doc(hidden)]
    #[allow(clippy::should_implement_trait)]
    pub fn into_iter(self) -> <Vec<u8> as IntoIterator>::IntoIter {
        self.bytes.into_iter()
    }
}

impl Debug for ByteString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.bytes, f)
    }
}

impl AsRef<[u8]> for ByteString {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl AsMut<[u8]> for ByteString {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
}

impl Deref for ByteString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl DerefMut for ByteString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bytes
    }
}

impl<Rhs> PartialEq<Rhs> for ByteString
where
    Rhs: ?Sized + AsRef<[u8]>,
{
    fn eq(&self, other: &Rhs) -> bool {
        self.as_ref().eq(other.as_ref())
    }
}

impl<Rhs> PartialOrd<Rhs> for ByteString
where
    Rhs: ?Sized + AsRef<[u8]>,
{
    fn partial_cmp(&self, other: &Rhs) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl Hash for ByteString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl IntoIterator for ByteString {
    type Item = u8;
    type IntoIter = <Vec<u8> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.bytes.into_iter()
    }
}

impl<'a> IntoIterator for &'a ByteString {
    type Item = &'a u8;
    type IntoIter = <&'a [u8] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.bytes.iter()
    }
}

impl<'a> IntoIterator for &'a mut ByteString {
    type Item = &'a mut u8;
    type IntoIter = <&'a mut [u8] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.bytes.iter_mut()
    }
}

impl Serialize for ByteString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.bytes)
    }
}

struct ByteStringVisitor;

impl<'de> Visitor<'de> for ByteStringVisitor {
    type Value = ByteString;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("byte array")
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<ByteString, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let len = cmp::min(visitor.size_hint().unwrap_or(0), 4096);
        let mut bytes = Vec::with_capacity(len);

        while let Some(b) = visitor.next_element()? {
            bytes.push(b);
        }

        Ok(ByteString::from(bytes))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<ByteString, E>
    where
        E: Error,
    {
        Ok(ByteString::from(v))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<ByteString, E>
    where
        E: Error,
    {
        Ok(ByteString::from(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<ByteString, E>
    where
        E: Error,
    {
        Ok(ByteString::from(v))
    }

    fn visit_string<E>(self, v: String) -> Result<ByteString, E>
    where
        E: Error,
    {
        Ok(ByteString::from(v))
    }
}

impl<'de> Deserialize<'de> for ByteString {
    fn deserialize<D>(deserializer: D) -> Result<ByteString, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_byte_buf(ByteStringVisitor)
    }
}
