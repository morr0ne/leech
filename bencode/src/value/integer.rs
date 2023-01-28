use serde::Serialize;
use std::fmt::Debug;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(PartialEq, Eq, PartialOrd, Ord)]
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

impl Serialize for Integer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.inner {
            IntegerType::Negative(integer) => serializer.serialize_i64(integer),
            IntegerType::Positive(integer) => serializer.serialize_u64(integer),
        }
    }
}
