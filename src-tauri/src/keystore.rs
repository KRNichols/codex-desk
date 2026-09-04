//! OS-backed DEK and optional PAT slot.
//!
//! Custody order:
//! 1. Windows: DPAPI (`CryptProtectData`) wrap of a random DEK, plus Credential Manager when available.
//! 2. Other OS: `keyring` (Credential Manager / Keychain / Secret Service / kernel keyutils).
//! 3. Fallback: machine-bound wrap (HKDF-SHA-256 over machine-id + OS user) in a 0600 file.
//!
//! The DEK is never committed. The PAT is never written to SQLite.

use crate::crypto::{self, Dek};
use hkdf::Hkdf;
use sha2::Sha256;
use std::fs;
use std::path::Path;
use zeroize::Zeroize;

const SERVICE: &str = "codex-desk";
const DEK_ACCOUNT: &str = "store-dek-v1";
const PAT_ACCOUNT: &str = "azure-llm-pat";
const WRAP_FILE: &str = "dek.wrap";
const PAT_WRAP_FILE: &str = "pat.wrap";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyBackend {
    #[allow(dead_code)]
    WindowsDpapi,
    OsKeyring,
    MachineBound,
}

impl KeyBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            KeyBackend::WindowsDpapi => "windows-dpapi",
            KeyBackend::OsKeyring => "os-keyring",
            KeyBackend::MachineBound => "machine-bound",
        }
    }
}

pub struct UnlockedKey {
    pub dek: Dek,
    pub backend: KeyBackend,
}

impl Drop for UnlockedKey {
    fn drop(&mut self) {
        self.dek.zeroize();
    }
}

pub fn load_or_create_dek(app_data: &Path) -> Result<UnlockedKey, String> {
    if let Some(existing) = try_load_dek(app_data) {
        return Ok(existing);
    }
    let dek = crypto::random_dek();
    persist_dek(app_data, &dek)
}

fn try_load_dek(app_data: &Path) -> Option<UnlockedKey> {
    #[cfg(windows)]
    {
        if let Ok(dek) = load_dpapi_wrap(app_data) {
            return Some(UnlockedKey {
                dek,
                backend: KeyBackend::WindowsDpapi,
            });
        }
    }
    if let Ok(dek) = load_keyring_dek() {
        return Some(UnlockedKey {
            dek,
            backend: KeyBackend::OsKeyring,
        });
    }
    if let Ok(dek) = load_machine_wrap(app_data) {
        return Some(UnlockedKey {
            dek,
            backend: KeyBackend::MachineBound,
        });
    }
    None
}

fn persist_dek(app_data: &Path, dek: &Dek) -> Result<UnlockedKey, String> {
    fs::create_dir_all(app_data).map_err(|e| format!("create app data: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(app_data, fs::Permissions::from_mode(0o700));
    }

    #[cfg(windows)]
    {
        if let Err(err) = save_dpapi_wrap(app_data, dek) {
            return Err(format!("DPAPI key wrap failed: {err}"));
        }
        let _ = save_keyring_dek(dek);
        return Ok(UnlockedKey {
            dek: *dek,
            backend: KeyBackend::WindowsDpapi,
        });
    }

    #[cfg(not(windows))]
    {
        if save_keyring_dek(dek).is_ok() {
            let _ = save_machine_wrap(app_data, dek);
            return Ok(UnlockedKey {
                dek: *dek,
                backend: KeyBackend::OsKeyring,
            });
        }
        save_machine_wrap(app_data, dek)?;
        Ok(UnlockedKey {
            dek: *dek,
            backend: KeyBackend::MachineBound,
        })
    }
}

fn load_keyring_dek() -> Result<Dek, String> {
    let entry = keyring::Entry::new(SERVICE, DEK_ACCOUNT).map_err(|e| e.to_string())?;
    let secret = entry.get_password().map_err(|e| e.to_string())?;
    decode_dek(&secret)
}

fn save_keyring_dek(dek: &Dek) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE, DEK_ACCOUNT).map_err(|e| e.to_string())?;
    entry.set_password(&hex::encode(dek)).map_err(|e| e.to_string())
}

fn load_machine_wrap(app_data: &Path) -> Result<Dek, String> {
    let path = app_data.join(WRAP_FILE);
    let blob = fs::read(&path).map_err(|e| format!("read dek.wrap: {e}"))?;
    let kek = machine_kek()?;
    let raw = crypto::open(&kek, &blob)?;
    decode_dek_bytes(&raw)
}

fn save_machine_wrap(app_data: &Path, dek: &Dek) -> Result<(), String> {
    let kek = machine_kek()?;
    let blob = crypto::seal(&kek, dek)?;
    let path = app_data.join(WRAP_FILE);
    fs::write(&path, blob).map_err(|e| format!("write dek.wrap: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

fn machine_kek() -> Result<Dek, String> {
    let ikm = format!(
        "{}|{}",
        crate::identity::machine_id(),
        crate::identity::session_user()
    );
    let hk = Hkdf::<Sha256>::new(Some(b"codex-desk-il5-store-v1"), ikm.as_bytes());
    let mut out = [0u8; 32];
    hk.expand(b"dek-wrap", &mut out)
        .map_err(|_| "HKDF expand failed".to_string())?;
    Ok(out)
}

fn decode_dek(hex_str: &str) -> Result<Dek, String> {
    let bytes = hex::decode(hex_str.trim()).map_err(|e| format!("dek hex: {e}"))?;
    decode_dek_bytes(&bytes)
}

fn decode_dek_bytes(bytes: &[u8]) -> Result<Dek, String> {
    if bytes.len() != 32 {
        return Err("dek is not 32 bytes".into());
    }
    let mut dek = [0u8; 32];
    dek.copy_from_slice(bytes);
    Ok(dek)
}

pub fn get_pat_slot(app_data: &Path) -> Result<Option<String>, String> {
    if let Ok(entry) = keyring::Entry::new(SERVICE, PAT_ACCOUNT) {
        if let Ok(value) = entry.get_password() {
            if !value.is_empty() {
                return Ok(Some(value));
            }
        }
    }
    let path = app_data.join(PAT_WRAP_FILE);
    if !path.is_file() {
        return Ok(None);
    }
    let unlocked = load_or_create_dek(app_data)?;
    let blob = fs::read(&path).map_err(|e| e.to_string())?;
    let raw = crypto::open(&unlocked.dek, &blob)?;
    let value = String::from_utf8(raw).map_err(|_| "pat slot is not utf8".to_string())?;
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

pub fn pat_slot_present(app_data: &Path) -> bool {
    if let Ok(entry) = keyring::Entry::new(SERVICE, PAT_ACCOUNT) {
        if entry.get_password().ok().filter(|v| !v.is_empty()).is_some() {
            return true;
        }
    }
    app_data.join(PAT_WRAP_FILE).is_file()
}

pub fn set_pat_slot(app_data: &Path, pat: &str) -> Result<String, String> {
    let trimmed = pat.trim();
    if trimmed.is_empty() {
        return Err("PAT is empty.".into());
    }
    if let Ok(entry) = keyring::Entry::new(SERVICE, PAT_ACCOUNT) {
        if entry.set_password(trimmed).is_ok() {
            return Ok("os-keyring".into());
        }
    }
    let unlocked = load_or_create_dek(app_data)?;
    let blob = crypto::seal(&unlocked.dek, trimmed.as_bytes())?;
    let path = app_data.join(PAT_WRAP_FILE);
    fs::write(&path, blob).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }
    Ok("os-secret-store".into())
}

pub fn clear_pat_slot(app_data: &Path) -> Result<(), String> {
    if let Ok(entry) = keyring::Entry::new(SERVICE, PAT_ACCOUNT) {
        let _ = entry.delete_credential();
    }
    let path = app_data.join(PAT_WRAP_FILE);
    if path.is_file() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(windows)]
fn save_dpapi_wrap(app_data: &Path, dek: &Dek) -> Result<(), String> {
    let protected = dpapi_protect(dek)?;
    let path = app_data.join("dek.dpapi");
    fs::write(&path, protected).map_err(|e| e.to_string())
}

#[cfg(windows)]
fn load_dpapi_wrap(app_data: &Path) -> Result<Dek, String> {
    let path = app_data.join("dek.dpapi");
    let blob = fs::read(&path).map_err(|e| e.to_string())?;
    let raw = dpapi_unprotect(&blob)?;
    decode_dek_bytes(&raw)
}

#[cfg(windows)]
fn dpapi_protect(plain: &[u8]) -> Result<Vec<u8>, String> {
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{CryptProtectData, CRYPT_INTEGER_BLOB};
    let input = CRYPT_INTEGER_BLOB {
        cbData: plain.len() as u32,
        pbData: plain.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    let ok = unsafe {
        CryptProtectData(
            &input,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            0,
            &mut output,
        )
    };
    if ok == 0 {
        return Err("CryptProtectData failed".into());
    }
    let slice = unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize) };
    let vec = slice.to_vec();
    unsafe {
        LocalFree(output.pbData as _);
    }
    Ok(vec)
}

#[cfg(windows)]
fn dpapi_unprotect(blob: &[u8]) -> Result<Vec<u8>, String> {
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB};
    let input = CRYPT_INTEGER_BLOB {
        cbData: blob.len() as u32,
        pbData: blob.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    let ok = unsafe {
        CryptUnprotectData(
            &input,
            std::ptr::null_mut(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            0,
            &mut output,
        )
    };
    if ok == 0 {
        return Err("CryptUnprotectData failed".into());
    }
    let slice = unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize) };
    let vec = slice.to_vec();
    unsafe {
        LocalFree(output.pbData as _);
    }
    Ok(vec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn machine_wrap_round_trip() {
        let dir = TempDir::new().unwrap();
        let dek = crypto::random_dek();
        save_machine_wrap(dir.path(), &dek).unwrap();
        assert_eq!(load_machine_wrap(dir.path()).unwrap(), dek);
    }
}
