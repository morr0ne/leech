pub mod decode;
pub mod encode;
mod token;

#[derive(Debug)]
pub struct AsString<I>(pub I);
