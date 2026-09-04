//! Local CUI envelope.
//!
//! Algorithm: AES-256-GCM (NIST SP 800-38D), 256-bit DEK, 96-bit random nonce.
//! File layout: `CDEX1` || version(1) || nonce(12) || ciphertext || tag(16).
//!
//! This crate is **not** a FIPS 140-3 validated module. Desk inherits FIPS
//! from the OS / Codex / Azure path. See SECURITY.md.

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;

pub const MAGIC: &[u8; 5] = b"CDEX1";
pub const VERSION: u8 = 1;
pub const NONCE_LEN: usize = 12;
pub const KEY_LEN: usize = 32;
pub const TAG_LEN: usize = 16;

pub type Dek = [u8; KEY_LEN];

pub fn random_dek() -> Dek {
    let mut key = [0u8; KEY_LEN];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

pub fn seal(key: &Dek, plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| "AES-256-GCM encrypt failed".to_string())?;
    let mut out = Vec::with_capacity(MAGIC.len() + 1 + NONCE_LEN + ct.len());
    out.extend_from_slice(MAGIC);
    out.push(VERSION);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    Ok(out)
}

pub fn open(key: &Dek, blob: &[u8]) -> Result<Vec<u8>, String> {
    let min = MAGIC.len() + 1 + NONCE_LEN + TAG_LEN;
    if blob.len() < min {
        return Err("encrypted store is truncated".into());
    }
    if &blob[..5] != MAGIC {
        return Err("not a Codex Desk encrypted store (missing CDEX1 magic)".into());
    }
    if blob[5] != VERSION {
        return Err(format!("unsupported store version {}", blob[5]));
    }
    let nonce = Nonce::from_slice(&blob[6..18]);
    let ct = &blob[18..];
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    cipher
        .decrypt(nonce, ct)
        .map_err(|_| "decrypt failed — key unlock failed or the file was tampered".to_string())
}

pub fn looks_like_envelope(blob: &[u8]) -> bool {
    blob.len() >= MAGIC.len() + 1 + NONCE_LEN + TAG_LEN && &blob[..5] == MAGIC
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let key = random_dek();
        let sealed = seal(&key, b"cui-transcript").unwrap();
        assert!(looks_like_envelope(&sealed));
        assert_eq!(open(&key, &sealed).unwrap(), b"cui-transcript");
    }

    #[test]
    fn wrong_key_fails() {
        let sealed = seal(&random_dek(), b"x").unwrap();
        assert!(open(&random_dek(), &sealed).is_err());
    }
}
