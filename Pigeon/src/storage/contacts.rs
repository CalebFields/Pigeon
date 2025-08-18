use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Contact {
    pub id: u64,
    pub name: String,
    pub addr: String,        // multiaddr string
    pub public_key: Vec<u8>, // 32 bytes (sodium box public key)
    pub ping_interval: u64,  // in seconds
}

#[allow(dead_code)]
pub struct ContactStore {
    db: sled::Db,
}

#[allow(dead_code)]
impl ContactStore {
    pub fn open_in_dir(data_dir: &Path) -> Result<Self, super::Error> {
        std::fs::create_dir_all(data_dir)
            .map_err(|e| super::Error::Serialization(e.to_string()))?;
        let path = data_dir.join("contacts_db");
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    pub fn add(
        &self,
        name: &str,
        addr: &str,
        public_key_hex: &str,
    ) -> Result<Contact, super::Error> {
        // Validate inputs
        let name = name.trim();
        if name.is_empty() {
            return Err(super::Error::Validation("name cannot be empty".into()));
        }
        if !addr.trim_start().starts_with('/') {
            return Err(super::Error::Validation(
                "addr must be a multiaddr starting with '/'".into(),
            ));
        }
        let public_key = hex::decode(public_key_hex)
            .map_err(|e| super::Error::Validation(format!("invalid pubkey hex: {e}")))?;
        if public_key.len() != 32 {
            return Err(super::Error::Validation(
                "pubkey must be 32 bytes (64 hex chars)".into(),
            ));
        }

        let id = self.db.generate_id()?;
        let contact = Contact {
            id,
            name: name.to_string(),
            addr: addr.to_string(),
            public_key,
            ping_interval: 0,
        };
        let key_bytes = id.to_be_bytes();
        // encrypt-at-rest
        let cfg = crate::config::load();
        let rest_key = super::at_rest::AtRestKey::load_or_create(&cfg.data_dir)?;
        let serialized =
            bincode::serialize(&contact).map_err(|e| super::Error::Serialization(e.to_string()))?;
        let sealed = super::at_rest::encrypt(&rest_key, &serialized)?;
        self.db.insert(key_bytes, sealed)?;
        Ok(contact)
    }

    pub fn get(&self, id: u64) -> Result<Option<Contact>, super::Error> {
        let id_bytes = id.to_be_bytes();
        if let Some(contact_bytes) = self.db.get(id_bytes)? {
            // decrypt-at-rest
            let cfg = crate::config::load();
            let key = super::at_rest::AtRestKey::load_or_create(&cfg.data_dir)?;
            let plain = super::at_rest::decrypt(&key, &contact_bytes)
                .map_err(|e| super::Error::Serialization(e.to_string()))?;
            let contact = bincode::deserialize::<Contact>(&plain)
                .map_err(|e| super::Error::Serialization(e.to_string()))?;
            Ok(Some(contact))
        } else {
            Ok(None)
        }
    }

    pub fn list(&self) -> Result<Vec<Contact>, super::Error> {
        let mut out = Vec::new();
        for item in self.db.iter() {
            let (_k, v) = item?;
            let cfg = crate::config::load();
            let key = super::at_rest::AtRestKey::load_or_create(&cfg.data_dir)?;
            let plain = super::at_rest::decrypt(&key, &v)
                .map_err(|e| super::Error::Serialization(e.to_string()))?;
            let contact = bincode::deserialize::<Contact>(&plain)
                .map_err(|e| super::Error::Serialization(e.to_string()))?;
            out.push(contact);
        }
        // stable ordering by id
        out.sort_by_key(|c| c.id);
        Ok(out)
    }

    pub fn remove(&self, id: u64) -> Result<bool, super::Error> {
        let existed = self.db.remove(id.to_be_bytes())?.is_some();
        Ok(existed)
    }

    pub fn update(
        &self,
        id: u64,
        name: &str,
        addr: &str,
        public_key_hex: &str,
    ) -> Result<Contact, super::Error> {
        // Validate inputs
        let name = name.trim();
        if name.is_empty() {
            return Err(super::Error::Validation("name cannot be empty".into()));
        }
        if !addr.trim_start().starts_with('/') {
            return Err(super::Error::Validation(
                "addr must be a multiaddr starting with '/'".into(),
            ));
        }
        let public_key = hex::decode(public_key_hex)
            .map_err(|e| super::Error::Validation(format!("invalid pubkey hex: {e}")))?;
        if public_key.len() != 32 {
            return Err(super::Error::Validation(
                "pubkey must be 32 bytes (64 hex chars)".into(),
            ));
        }

        // Keep ping_interval if existing
        let prev = self.get(id)?;
        let ping_interval = prev.map(|c| c.ping_interval).unwrap_or(0);

        let contact = Contact {
            id,
            name: name.to_string(),
            addr: addr.to_string(),
            public_key,
            ping_interval,
        };
        let key_bytes = id.to_be_bytes();
        let cfg = crate::config::load();
        let rest_key = super::at_rest::AtRestKey::load_or_create(&cfg.data_dir)?;
        let serialized =
            bincode::serialize(&contact).map_err(|e| super::Error::Serialization(e.to_string()))?;
        let sealed = super::at_rest::encrypt(&rest_key, &serialized)?;
        self.db.insert(key_bytes, sealed)?;
        Ok(contact)
    }

    pub fn find_by_name_case_insensitive(
        &self,
        name: &str,
    ) -> Result<Option<Contact>, super::Error> {
        let needle = name.to_ascii_lowercase();
        for c in self.list()? {
            if c.name.to_ascii_lowercase() == needle {
                return Ok(Some(c));
            }
        }
        Ok(None)
    }
}
