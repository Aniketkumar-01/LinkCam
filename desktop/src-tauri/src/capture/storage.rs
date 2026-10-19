use chrono::Local;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub struct CaptureStorage {
    base_dir: PathBuf,
}

impl CaptureStorage {
    pub fn new() -> Self {
        let base_dir = dirs_base().join("LinkCam");
        if let Err(e) = fs::create_dir_all(&base_dir) {
            eprintln!("[LinkCam] Warning: could not create capture directory: {:?}", e);
        }
        Self { base_dir }
    }

    pub fn get_base_dir(&self) -> &Path {
        &self.base_dir
    }

    pub fn save_capture(&self, file_name: &str, data: &[u8]) -> io::Result<PathBuf> {
        let clean_name = sanitize_filename(file_name);
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let safe_name = format!("{}_{}", timestamp, clean_name);
        let target_path = self.base_dir.join(&safe_name);

        // Security check: ensure target_path is strictly within base_dir
        if !target_path.starts_with(&self.base_dir) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Path traversal attempt detected",
            ));
        }

        let mut file = File::create(&target_path)?;
        file.write_all(data)?;
        println!("[LinkCam] Saved instant capture to {:?}", target_path);

        Ok(target_path)
    }
}

/// Sanitize untrusted filename received from network packets:
/// 1. Strips any directory traversal components (`..`, `/`, `\`)
/// 2. Whitelists only safe alphanumeric, '.', '_', '-' characters
/// 3. Rejects Windows DOS reserved device names (CON, PRN, AUX, NUL, COM1-9, LPT1-9)
fn sanitize_filename(input: &str) -> String {
    let base = Path::new(input)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("capture.bin");

    let filtered: String = base
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '_' || *c == '-')
        .collect();

    let stem = Path::new(&filtered)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_uppercase();

    let is_reserved = matches!(
        stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "COM1" | "COM2" | "COM3" | "COM4" | "COM5" | "COM6"
            | "COM7" | "COM8" | "COM9" | "LPT1" | "LPT2" | "LPT3" | "LPT4" | "LPT5"
            | "LPT6" | "LPT7" | "LPT8" | "LPT9"
    );

    if filtered.is_empty() || filtered.starts_with('.') || is_reserved {
        "capture.bin".to_string()
    } else {
        filtered
    }
}

fn dirs_base() -> PathBuf {
    if let Ok(profile) = std::env::var("USERPROFILE") {
        PathBuf::from(profile).join("Pictures")
    } else {
        std::env::temp_dir()
    }
}
