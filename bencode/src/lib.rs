// #![warn(
//     clippy::all,
//     clippy::restriction,
//     clippy::pedantic,
//     clippy::nursery,
//     clippy::cargo,
// )]

mod de;
mod error;
mod ser;
mod value;

pub use de::{from_bytes, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_vec, to_writer, Serializer};
pub use value::{Integer, Value};
