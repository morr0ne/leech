// #![warn(
//     clippy::all,
//     clippy::restriction,
//     clippy::pedantic,
//     clippy::nursery,
//     clippy::cargo,
// )]

mod byte_string;
mod de;
mod error;
mod ser;
pub mod value;

pub use byte_string::ByteString;
pub use de::{from_bytes, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_bytes, to_writer, Serializer};
pub use value::{Integer, Value};
