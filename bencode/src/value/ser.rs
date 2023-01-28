use serde::Serialize;

use crate::value::Value;

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::ByteString(byte_string) => byte_string.serialize(serializer),
            Value::Integer(integer) => integer.serialize(serializer),
            Value::List(list) => list.serialize(serializer),
            Value::Dictionary(dictionary) => dictionary.serialize(serializer),
        }
    }
}
