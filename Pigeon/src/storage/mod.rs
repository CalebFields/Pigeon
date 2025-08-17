pub mod contacts;
pub mod queue;

#[allow(unused_imports)]
pub use contacts::ContactStore;
#[allow(unused_imports)]
pub use queue::MessageQueue;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Db(#[from] sled::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Contact not found: {0}")]
    ContactNotFound(String),
}