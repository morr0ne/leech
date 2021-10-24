#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unexpected field {0}")]
    UnexpectedField(String),
    #[error("Unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, Error>;
