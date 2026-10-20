use std::fs;
use std::path::PathBuf;

/// Path to local AppData directory for LinkCam trusted tokens
fn get_trusted_store_path() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    let dir = base.join("LinkCam");
    let _ = fs::create_dir_all(&dir);
    dir.join("trusted_devices.dat")
}

#[cfg(windows)]
pub fn encrypt_bytes(plaintext: &[u8]) -> Result<Vec<u8>, String> {
    use windows::Win32::Security::Cryptography::{
        CryptProtectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let mut in_blob = CRYPT_INTEGER_BLOB {
        cbData: plaintext.len() as u32,
        pbData: plaintext.as_ptr() as *mut u8,
    };
    let mut out_blob = CRYPT_INTEGER_BLOB::default();

    unsafe {
        CryptProtectData(
            &mut in_blob,
            windows::core::PCWSTR::null(),
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut out_blob,
        )
        .map_err(|e| format!("CryptProtectData failed: {:?}", e))?;

        let slice = std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize);
        let result = slice.to_vec();
        windows::Win32::Foundation::LocalFree(out_blob.pbData as _);
        Ok(result)
    }
}

#[cfg(not(windows))]
pub fn encrypt_bytes(plaintext: &[u8]) -> Result<Vec<u8>, String> {
    // Fallback for non-Windows platforms/testing
    Ok(plaintext.to_vec())
}

#[cfg(windows)]
pub fn decrypt_bytes(ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    use windows::Win32::Security::Cryptography::{
        CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let mut in_blob = CRYPT_INTEGER_BLOB {
        cbData: ciphertext.len() as u32,
        pbData: ciphertext.as_ptr() as *mut u8,
    };
    let mut out_blob = CRYPT_INTEGER_BLOB::default();

    unsafe {
        CryptUnprotectData(
            &mut in_blob,
            None,
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut out_blob,
        )
        .map_err(|e| format!("CryptUnprotectData failed: {:?}", e))?;

        let slice = std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize);
        let result = slice.to_vec();
        windows::Win32::Foundation::LocalFree(out_blob.pbData as _);
        Ok(result)
    }
}

#[cfg(not(windows))]
pub fn decrypt_bytes(ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    // Fallback for non-Windows platforms/testing
    Ok(ciphertext.to_vec())
}

/// Save an authenticated device token into Windows DPAPI protected store
pub fn save_trusted_device(device_id: &str, token: &str) -> Result<(), String> {
    let record = format!("{}:{}\n", device_id, token);
    let encrypted = encrypt_bytes(record.as_bytes())?;
    let path = get_trusted_store_path();
    fs::write(path, encrypted).map_err(|e| format!("Failed to write trusted device store: {:?}", e))
}

/// Check if an incoming device_id and token match a stored trusted credential
pub fn is_device_trusted(device_id: &str, token: &str) -> bool {
    let path = get_trusted_store_path();
    if !path.exists() {
        return false;
    }

    if let Ok(encrypted) = fs::read(&path) {
        if let Ok(decrypted) = decrypt_bytes(&encrypted) {
            let content = String::from_utf8_lossy(&decrypted);
            let expected = format!("{}:{}", device_id, token);
            return content.lines().any(|line| line == expected);
        }
    }

    false
}

/// Revoke all trusted devices by deleting the DPAPI store
pub fn revoke_all_trusted_devices() -> Result<(), String> {
    let path = get_trusted_store_path();
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("Failed to delete trusted store: {:?}", e))?;
    }
    Ok(())
}
