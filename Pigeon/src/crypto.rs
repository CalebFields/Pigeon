use sodiumoxide::crypto::{box_, secretbox, sign};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Encryption failed: {0}")]
    Encryption(String),
    #[error("Decryption failed: {0}")]
    Decryption(String),
    #[error("Key generation failed: {0}")]
    KeyGeneration(String),
    #[error("Signature verification failed: {0}")]
    Signature(String),
}

pub struct KeyPair {
    pub public: box_::PublicKey,
    pub secret: box_::SecretKey,
}

impl KeyPair {
    pub fn generate() -> Self {
        let (public, secret) = box_::gen_keypair();
        Self { public, secret }
    }
}

pub fn derive_auth_token(local_secret: &[u8], remote_pubkey: &[u8]) -> [u8; 16] {
    use blake2::{Blake2bMac, Digest};
    let mut h = Blake2bMac::new_from_slice(local_secret).expect("Valid key size");
    h.update(remote_pubkey);
    let result = h.finalize();
    let mut token = [0u8; 16];
    token.copy_from_slice(&result[..16]);
    token
}

pub fn encrypt_message(
    sender_priv: &box_::SecretKey,
    receiver_pub: &box_::PublicKey,
    plaintext: &[u8],
) -> Result<Vec<u8>, Error> {
    let nonce = box_::gen_nonce();
    let ciphertext = box_::seal(plaintext, &nonce, receiver_pub, sender_priv);
    Ok([nonce.as_ref(), &ciphertext].concat())
}

pub fn decrypt_message(
    ciphertext: &[u8],
    sender_pub: &box_::PublicKey,
    receiver_priv: &box_::SecretKey,
) -> Result<Vec<u8>, Error> {
    if ciphertext.len() < box_::NONCEBYTES {
        return Err(Error::Decryption("Ciphertext too short".into()));
    }
    
    let nonce = box_::Nonce::from_slice(&ciphertext[..box_::NONCEBYTES])
        .ok_or_else(|| Error::Decryption("Invalid nonce".into()))?;
    
    box_::open(
        &ciphertext[box_::NONCEBYTES..],
        &nonce,
        sender_pub,
        receiver_priv,
    )
    .map_err(|_| Error::Decryption("Decryption failed".into()))
}