use indexmap::IndexMap;
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
