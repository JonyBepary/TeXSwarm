use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("CRDT error: {0}")]
    CrdtError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Git error: {0}")]
    GitError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),

    #[error("Repository not found for document: {0}")]
    RepositoryNotFound(uuid::Uuid),

    #[error("Document not found: {0}")]
    DocumentNotFound(uuid::Uuid),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
