use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Debug)]
pub struct Contact {
    pub id: u64,
    pub name: String,
    pub addr: SocketAddr,
    pub public_key: Vec<u8>,
    pub ping_interval: u64, // in seconds
    pub auth_token: [u8; 16],
}

#[allow(dead_code)]
pub struct ContactStore {
    db: sled::Db,
}

#[allow(dead_code)]
impl ContactStore {
    pub fn new(path: &str) -> Result<Self, super::Error> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    pub fn add_contact(&self, contact: Contact) -> Result<(), super::Error> {
        let id_bytes = contact.id.to_be_bytes();
        let contact_bytes = bincode::serialize(&contact)
            .map_err(|e| super::Error::Serialization(e.to_string()))?;
        self.db.insert(id_bytes, contact_bytes)?;
        Ok(())
    }

    pub fn get_contact(&self, id: u64) -> Result<Option<Contact>, super::Error> {
        let id_bytes = id.to_be_bytes();
        if let Some(contact_bytes) = self.db.get(id_bytes)? {
            let contact = bincode::deserialize::<Contact>(&contact_bytes)
                .map_err(|e| super::Error::Serialization(e.to_string()))?;
            Ok(Some(contact))
        } else {
            Ok(None)
        }
    }
}