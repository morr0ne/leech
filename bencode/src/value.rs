use indexmap::IndexMap;
use serde_bytes::ByteBuf;

pub enum Value {
    ByteString(ByteBuf),
    Integer(Integer),
    List(Vec<Value>),
    Dictionary(IndexMap<ByteBuf, Value>),
}

pub struct Integer {
    inner: IntegerType,
}

enum IntegerType {
    Negative(i64),
    Positive(u64),
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
