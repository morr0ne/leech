// #![warn(
//     clippy::all,
//     clippy::restriction,
//     clippy::pedantic,
//     clippy::nursery,
//     clippy::cargo,
// )]

mod de;
mod error;
mod value;

pub use de::{from_bytes, Deserializer};
pub use error::{Error, Result};
pub use value::{Integer, Value};
// pub use crate::ser::{to_string, Serializer};
