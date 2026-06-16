use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaoError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("storage engine error: {0}")]
    Storage(#[from] sled::Error),

    #[error("hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("data unit not found: {0}")]
    NotFound(String),

    #[error("insufficient fragments: have {have}, need {need}")]
    InsufficientFragments { have: usize, need: usize },

    #[error("qi state error: {0}")]
    QiState(String),

    #[error("crypto error: {0}")]
    Crypto(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("consensus error: {0}")]
    Consensus(String),

    #[error("wasm error: {0}")]
    Wasm(String),
}

pub type TaoResult<T> = Result<T, TaoError>;
