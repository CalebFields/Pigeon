use sodiumoxide::crypto::secretbox;
use std::fs;
use std::path::Path;

const KEY_FILE: &str = "at_rest.key";

pub struct AtRestKey(secretbox::Key);

impl AtRestKey {
    pub fn load_or_create(data_dir: &Path) -> Result<Self, super::Error> {
        fs::create_dir_all(data_dir).map_err(|e| super::Error::Serialization(e.to_string()))?;
        let path = data_dir.join(KEY_FILE);
        if let Ok(bytes) = fs::read(&path) {
            let mut arr = [0u8; secretbox::KEYBYTES];
            if bytes.len() != arr.len() { return Err(super::Error::Crypto("invalid key length".into())); }
            arr.copy_from_slice(&bytes);
            Ok(Self(secretbox::Key(arr)))
        } else {
            let key = secretbox::gen_key();
            fs::write(&path, key.0.as_slice()).map_err(|e| super::Error::Serialization(e.to_string()))?;
            Ok(Self(key))
        }
    }

    pub fn key(&self) -> &secretbox::Key { &self.0 }
}

pub fn encrypt(key: &AtRestKey, plaintext: &[u8]) -> Result<Vec<u8>, super::Error> {
    let nonce = secretbox::gen_nonce();
    let mut out = Vec::with_capacity(secretbox::NONCEBYTES + plaintext.len() + secretbox::MACBYTES);
    out.extend_from_slice(nonce.0.as_slice());
    let ct = secretbox::seal(plaintext, &nonce, key.key());
    out.extend_from_slice(&ct);
    Ok(out)
}

pub fn decrypt(key: &AtRestKey, ciphertext: &[u8]) -> Result<Vec<u8>, super::Error> {
    if ciphertext.len() < secretbox::NONCEBYTES + secretbox::MACBYTES {
        return Err(super::Error::Crypto("ciphertext too short".into()));
    }
    let mut nonce_bytes = [0u8; secretbox::NONCEBYTES];
    nonce_bytes.copy_from_slice(&ciphertext[..secretbox::NONCEBYTES]);
    let nonce = secretbox::Nonce(nonce_bytes);
    let pt = secretbox::open(&ciphertext[secretbox::NONCEBYTES..], &nonce, key.key())
        .map_err(|_| super::Error::Crypto("decryption failed".into()))?;
    Ok(pt)
}


