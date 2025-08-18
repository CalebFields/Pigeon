use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Network error: {0}")]
    #[cfg(feature = "network")]
    Network(#[from] crate::network::Error),

    #[error("Crypto error: {0}")]
    Crypto(#[from] crate::crypto::Error),

    #[error("Storage error: {0}")]
    Storage(#[from] crate::storage::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    #[allow(dead_code)]
    Serialization(String),
}

#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, Error>;
