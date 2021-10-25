#[derive(Debug, thiserror::Error)]
pub enum EncodingError {
    #[error("Unknown error")]
    Unknown,
}
