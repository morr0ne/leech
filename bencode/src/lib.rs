pub mod decode;
pub mod encode;
pub mod error;

#[derive(Debug)]
pub enum Value<'a> {
    ByteString(&'a [u8]),
    Integer(i64),
    List(Vec<Value<'a>>),
    Dictionary(indexmap::IndexMap<&'a [u8], Value<'a>>),
}
