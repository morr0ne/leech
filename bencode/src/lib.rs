use indexmap::IndexMap;
pub mod decode;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Value<'a> {
    ByteString(&'a [u8]),
    Integer(i64),
    List(Vec<Value<'a>>),
    Dictionary(IndexMap<&'a [u8], Value<'a>>),
}

pub struct Decoder {
    data: Vec<u8>,
}

impl Decoder {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}
