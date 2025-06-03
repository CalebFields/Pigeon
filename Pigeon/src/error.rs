use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Network error: {0}")]
    Network(#[from] network::Error),
    
    #[error("Crypto error: {0}")]
    Crypto(#[from] crypto::Error),
    
    #[error("Storage error: {0}")]
    Storage(#[from] storage::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type Result<T> = std::result::Result<T, Error>;