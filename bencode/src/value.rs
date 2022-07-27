use crate::byte_string::ByteString;
use serde::{de::Visitor, Deserialize};
use std::collections::BTreeMap;
use std::fmt::Debug;

pub enum Value {
    ByteString(ByteString),
    Integer(Integer),
    List(Vec<Value>),
    Dictionary(BTreeMap<ByteString, Value>),
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ByteString(value) => f
                .debug_tuple("ByteString")
                .field(&String::from_utf8_lossy(value))
                .finish(),
            Self::Integer(value) => f.debug_tuple("Integer").field(value).finish(),
            Self::List(value) => {
                f.write_str("List(")?;
                Debug::fmt(value, f)?;
                f.write_str(")")
            }
            Self::Dictionary(value) => {
                f.write_str("Dictionary(")?;
                Debug::fmt(value, f)?;
                f.write_str(")")
            }
        }
    }
}

pub struct Integer {
    inner: IntegerType,
}

impl Debug for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // f.debug_struct("Integer").field("inner", &self.inner).finish()
        match self.inner {
            IntegerType::Negative(n) => write!(f, "{n}"),
            IntegerType::Positive(n) => write!(f, "{n}"),
        }
    }
}

enum IntegerType {
    Negative(i64),
    Positive(u64),
}

impl From<i64> for Integer {
    fn from(value: i64) -> Self {
        Self {
            inner: IntegerType::Negative(value),
        }
    }
}

impl From<u64> for Integer {
    fn from(value: u64) -> Self {
        Self {
            inner: IntegerType::Positive(value),
        }
    }
}

impl Integer {
    pub fn is_i64(&self) -> bool {
        match self.inner {
            IntegerType::Positive(n) => n <= i64::max_value() as u64,
            IntegerType::Negative(_) => true,
        }
    }

    pub fn is_u64(&self) -> bool {
        matches!(self.inner, IntegerType::Positive(_))
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self.inner {
            IntegerType::Negative(n) => Some(n),
            IntegerType::Positive(n) => {
                if n <= i64::MAX as u64 {
                    Some(n as i64)
                } else {
                    None
                }
            }
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self.inner {
            IntegerType::Positive(n) => Some(n),
            IntegerType::Negative(_) => None,
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("any valid Bencode value")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Integer(value.into()))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Integer(value.into()))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::ByteString(ByteString::from(value)))
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::ByteString(ByteString::from(value)))
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = seq.next_element()? {
                    vec.push(elem);
                }

                Ok(Value::List(vec))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut dict = BTreeMap::new();

                while let Some((key, value)) = map.next_entry()? {
                    dict.insert(key, value);
                }

                Ok(Value::Dictionary(dict))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

#[cfg(test)]
mod tests {
    
}