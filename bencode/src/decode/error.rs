#[derive(Debug, thiserror::Error)]
pub enum DecodingError {
    #[error("Missing field {field}")]
    MissingField {
        field: String
    },
    #[error("Unexpected field {field}")]
    UnexpectedField {
        field: String
    },
    #[error("Unknown error")]
    Unknown,
}
