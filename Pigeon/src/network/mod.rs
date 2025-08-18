pub mod ping;
pub mod protocol;
pub mod rr;

pub use ping::NetworkManager;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Connection failed: {0}")]
    Connection(String),

    #[error("Handshake failed: {0}")]
    Handshake(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Address error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
