use indexmap::IndexMap;

#[derive(Debug)]
pub enum Value<'a> {
    ByteString(&'a [u8]),
    Integer(i64),
    List(Vec<Value<'a>>),
    Dictionary(IndexMap<&'a [u8], Value<'a>>),
}
