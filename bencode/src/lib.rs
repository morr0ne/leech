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

#[doc(inline)]
pub use byte_string::ByteString;
#[doc(inline)]
pub use de::{from_bytes, Deserializer};
#[doc(inline)]
pub use error::{Error, Result};
#[doc(inline)]
pub use ser::{to_bytes, to_writer, Serializer};
#[doc(inline)]
pub use value::{Integer, Value};
