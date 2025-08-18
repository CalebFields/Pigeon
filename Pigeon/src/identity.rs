use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[cfg(feature = "network")]
use libp2p::identity;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Identity {
    #[cfg(feature = "network")]
    pub libp2p: identity::Keypair,
    #[allow(dead_code)]
    pub sodium_box_pk: sodiumoxide::crypto::box_::PublicKey,
    #[allow(dead_code)]
    pub sodium_box_sk: sodiumoxide::crypto::box_::SecretKey,
    #[allow(dead_code)]
    pub sign_pk: sodiumoxide::crypto::sign::PublicKey,
    #[allow(dead_code)]
    pub sign_sk: sodiumoxide::crypto::sign::SecretKey,
}

#[derive(Serialize, Deserialize)]
struct StoredIdentity {
    libp2p_ed25519: Vec<u8>,
    sodium_box_pk: Vec<u8>,
    sodium_box_sk: Vec<u8>,
    sign_pk: Vec<u8>,
    sign_sk: Vec<u8>,
}

impl Identity {
    pub fn load_or_generate(data_dir: &Path) -> Result<Self, crate::error::Error> {
        fs::create_dir_all(data_dir).map_err(|e| crate::error::Error::Config(e.to_string()))?;
        let path = data_dir.join("identity.bin");
        if path.exists() {
            let mut f =
                fs::File::open(&path).map_err(|e| crate::error::Error::Config(e.to_string()))?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)
                .map_err(|e| crate::error::Error::Config(e.to_string()))?;
            let stored: StoredIdentity = bincode::deserialize(&buf)
                .map_err(|e| crate::error::Error::Config(e.to_string()))?;

            #[cfg(feature = "network")]
            let libp2p = {
                if !stored.libp2p_ed25519.is_empty() {
                    let mut bytes = stored.libp2p_ed25519.clone();
                    let ed = identity::ed25519::Keypair::try_from_bytes(bytes.as_mut_slice())
                        .map_err(|e| crate::error::Error::Config(format!("ed25519 decode: {e}")))?;
                    identity::Keypair::from(ed)
                } else {
                    let ed = identity::ed25519::Keypair::generate();
                    identity::Keypair::from(ed)
                }
            };

            let sodium_box_pk =
                sodiumoxide::crypto::box_::PublicKey::from_slice(&stored.sodium_box_pk)
                    .ok_or_else(|| crate::error::Error::Config("invalid sodium pk".to_string()))?;
            let sodium_box_sk =
                sodiumoxide::crypto::box_::SecretKey::from_slice(&stored.sodium_box_sk)
                    .ok_or_else(|| crate::error::Error::Config("invalid sodium sk".to_string()))?;
            let sign_pk = sodiumoxide::crypto::sign::PublicKey::from_slice(&stored.sign_pk)
                .ok_or_else(|| crate::error::Error::Config("invalid sign pk".to_string()))?;
            let sign_sk = sodiumoxide::crypto::sign::SecretKey::from_slice(&stored.sign_sk)
                .ok_or_else(|| crate::error::Error::Config("invalid sign sk".to_string()))?;

            Ok(Self {
                #[cfg(feature = "network")]
                libp2p,
                sodium_box_pk,
                sodium_box_sk,
                sign_pk,
                sign_sk,
            })
        } else {
            // Generate fresh identity
            #[cfg(feature = "network")]
            let ed = identity::ed25519::Keypair::generate();
            #[cfg(feature = "network")]
            let libp2p = identity::Keypair::from(ed.clone());
            #[cfg(feature = "network")]
            let ed_bytes: Vec<u8> = ed.to_bytes().to_vec();
            #[cfg(not(feature = "network"))]
            let ed_bytes: Vec<u8> = Vec::new();
            let (sodium_box_pk, sodium_box_sk) = sodiumoxide::crypto::box_::gen_keypair();
            let (sign_pk, sign_sk) = sodiumoxide::crypto::sign::gen_keypair();

            let stored = StoredIdentity {
                libp2p_ed25519: ed_bytes,
                sodium_box_pk: sodium_box_pk.0.to_vec(),
                sodium_box_sk: sodium_box_sk.0.to_vec(),
                sign_pk: sign_pk.0.to_vec(),
                sign_sk: sign_sk.0.to_vec(),
            };
            let bytes = bincode::serialize(&stored)
                .map_err(|e| crate::error::Error::Config(e.to_string()))?;
            let mut f =
                fs::File::create(&path).map_err(|e| crate::error::Error::Config(e.to_string()))?;
            // Best-effort restrictive perms
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
            }
            f.write_all(&bytes)
                .map_err(|e| crate::error::Error::Config(e.to_string()))?;
            Ok(Self {
                #[cfg(feature = "network")]
                libp2p,
                sodium_box_pk,
                sodium_box_sk,
                sign_pk,
                sign_sk,
            })
        }
    }

    #[allow(dead_code)]
    pub fn file_path(base: &Path) -> PathBuf {
        base.join("identity.bin")
    }
}
