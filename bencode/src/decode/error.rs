#[derive(Debug, thiserror::Error)]
pub enum DecodingError {
    #[error("Unexpected field {0}")]
    UnexpectedField(String),
    #[error("Unknown error")]
    Unknown,
}
