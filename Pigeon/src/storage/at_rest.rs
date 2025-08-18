use argon2::Argon2;
use once_cell::sync::Lazy;
use rand::RngCore;
use sodiumoxide::crypto::secretbox;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const KEY_FILE: &str = "at_rest.key";
const ENC_KEY_FILE: &str = "at_rest.key.enc";
const MAGIC: &[u8; 4] = b"PGN1"; // Pigeon v1

pub struct AtRestKey(secretbox::Key);

impl AtRestKey {
    pub fn load_or_create(data_dir: &Path) -> Result<Self, super::Error> {
        fs::create_dir_all(data_dir).map_err(|e| super::Error::Serialization(e.to_string()))?;
        // Return cached key if present for this data_dir (unlocked session)
        if let Some(k) = get_cached_key_for(data_dir) {
            return Ok(Self(k));
        }

        // If encrypted keyfile exists, try env-based unlock else report locked
        let enc_path = data_dir.join(ENC_KEY_FILE);
        if enc_path.exists() {
            if let Ok(pass) = std::env::var("PIGEON_PASSPHRASE") {
                return unlock_with_passphrase(data_dir, &pass);
            }
            return Err(super::Error::Crypto(
                "at-rest key is locked; run `security unlock` or set PIGEON_PASSPHRASE".into(),
            ));
        }

        let path = data_dir.join(KEY_FILE);
        if let Ok(bytes) = fs::read(&path) {
            let mut arr = [0u8; secretbox::KEYBYTES];
            if bytes.len() != arr.len() {
                return Err(super::Error::Crypto("invalid key length".into()));
            }
            arr.copy_from_slice(&bytes);
            Ok(Self(secretbox::Key(arr)))
        } else {
            let key = secretbox::gen_key();
            fs::write(&path, key.0.as_slice())
                .map_err(|e| super::Error::Serialization(e.to_string()))?;
            Ok(Self(key))
        }
    }

    pub fn key(&self) -> &secretbox::Key {
        &self.0
    }
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

// Optional: derive a key from passphrase using Argon2id
pub fn derive_key_from_passphrase(
    passphrase: &str,
    salt: &[u8],
) -> Result<secretbox::Key, super::Error> {
    let mut out = [0u8; secretbox::KEYBYTES];
    let params = Argon2::default();
    params
        .hash_password_into(passphrase.as_bytes(), salt, &mut out)
        .map_err(|e| super::Error::Crypto(format!("kdf: {e}")))?;
    Ok(secretbox::Key(out))
}

static CACHED_KEYS: Lazy<Mutex<HashMap<PathBuf, secretbox::Key>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn get_cached_key_for(data_dir: &Path) -> Option<secretbox::Key> {
    if let Ok(guard) = CACHED_KEYS.lock() {
        return guard.get(data_dir).cloned();
    }
    None
}

fn cache_key_for(data_dir: &Path, key: &secretbox::Key) {
    if let Ok(mut guard) = CACHED_KEYS.lock() {
        guard.insert(data_dir.to_path_buf(), key.clone());
    }
}

pub fn unlock_with_passphrase(
    data_dir: &Path,
    passphrase: &str,
) -> Result<AtRestKey, super::Error> {
    let enc_path = data_dir.join(ENC_KEY_FILE);
    let bytes = fs::read(&enc_path).map_err(|e| super::Error::Serialization(e.to_string()))?;
    if bytes.len() < 4 + 16 + secretbox::NONCEBYTES {
        return Err(super::Error::Crypto("enc keyfile too short".into()));
    }
    if &bytes[0..4] != MAGIC {
        return Err(super::Error::Crypto("enc keyfile bad magic".into()));
    }
    let salt = &bytes[4..20];
    let nonce_bytes = &bytes[20..20 + secretbox::NONCEBYTES];
    let mut nonce_arr = [0u8; secretbox::NONCEBYTES];
    nonce_arr.copy_from_slice(nonce_bytes);
    let nonce = secretbox::Nonce(nonce_arr);
    let sealed = &bytes[20 + secretbox::NONCEBYTES..];
    let kek = derive_key_from_passphrase(passphrase, salt)?;
    let key_bytes = secretbox::open(sealed, &nonce, &kek)
        .map_err(|_| super::Error::Crypto("unlock failed".into()))?;
    if key_bytes.len() != secretbox::KEYBYTES {
        return Err(super::Error::Crypto("bad inner key size".into()));
    }
    let mut arr = [0u8; secretbox::KEYBYTES];
    arr.copy_from_slice(&key_bytes);
    let key = secretbox::Key(arr);
    cache_key_for(data_dir, &key);
    Ok(AtRestKey(key))
}

pub fn set_passphrase_and_seal(data_dir: &Path, passphrase: &str) -> Result<(), super::Error> {
    fs::create_dir_all(data_dir).map_err(|e| super::Error::Serialization(e.to_string()))?;
    // Load existing key or generate
    let plain_path = data_dir.join(KEY_FILE);
    let key = if let Ok(bytes) = fs::read(&plain_path) {
        let mut arr = [0u8; secretbox::KEYBYTES];
        if bytes.len() != arr.len() {
            return Err(super::Error::Crypto("invalid key length".into()));
        }
        arr.copy_from_slice(&bytes);
        secretbox::Key(arr)
    } else if let Some(k) = get_cached_key_for(data_dir) {
        k
    } else {
        secretbox::gen_key()
    };

    // Derive KEK
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    let kek = derive_key_from_passphrase(passphrase, &salt)?;
    let nonce = secretbox::gen_nonce();
    let sealed = secretbox::seal(key.0.as_slice(), &nonce, &kek);

    // Write enc file
    let mut out = Vec::with_capacity(4 + salt.len() + secretbox::NONCEBYTES + sealed.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&salt);
    out.extend_from_slice(nonce.0.as_slice());
    out.extend_from_slice(&sealed);
    let enc_path = data_dir.join(ENC_KEY_FILE);
    fs::write(&enc_path, &out).map_err(|e| super::Error::Serialization(e.to_string()))?;
    // Remove plaintext key if exists
    let _ = fs::remove_file(&plain_path);
    cache_key_for(data_dir, &key);
    Ok(())
}

/// Rotate the at-rest key by generating a fresh key and sealing it with the given passphrase.
pub fn rotate_key_and_seal(data_dir: &Path, passphrase: &str) -> Result<(), super::Error> {
    fs::create_dir_all(data_dir).map_err(|e| super::Error::Serialization(e.to_string()))?;
    // Generate fresh key
    let key = secretbox::gen_key();

    // Derive KEK
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    let kek = derive_key_from_passphrase(passphrase, &salt)?;
    let nonce = secretbox::gen_nonce();
    let sealed = secretbox::seal(key.0.as_slice(), &nonce, &kek);

    // Write enc file
    let mut out = Vec::with_capacity(4 + salt.len() + secretbox::NONCEBYTES + sealed.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&salt);
    out.extend_from_slice(nonce.0.as_slice());
    out.extend_from_slice(&sealed);
    let enc_path = data_dir.join(ENC_KEY_FILE);
    fs::write(&enc_path, &out).map_err(|e| super::Error::Serialization(e.to_string()))?;
    // Remove plaintext key if exists
    let _ = fs::remove_file(data_dir.join(KEY_FILE));
    cache_key_for(data_dir, &key);
    Ok(())
}

#[cfg(test)]
pub fn test_clear_cache() {
    if let Ok(mut guard) = CACHED_KEYS.lock() {
        guard.clear();
    }
}
