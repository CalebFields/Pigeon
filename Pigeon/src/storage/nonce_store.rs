use sled::Tree;
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
pub struct NonceStore {
    tree: Tree,
}

#[allow(dead_code)]
impl NonceStore {
    pub fn open(db: &sled::Db) -> Result<Self, super::Error> {
        let tree = db.open_tree("nonces").map_err(super::Error::Db)?;
        Ok(Self { tree })
    }

    pub fn insert_if_fresh(&self, sender_id: u64, nonce: &[u8]) -> Result<bool, super::Error> {
        let mut key = sender_id.to_be_bytes().to_vec();
        key.extend_from_slice(nonce);
        // if exists -> replay
        if self.tree.contains_key(&key).map_err(super::Error::Db)? {
            return Ok(false);
        }
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let ts = now.to_be_bytes();
        self.tree.insert(key, ts.as_slice()).map_err(super::Error::Db)?;
        Ok(true)
    }
}


