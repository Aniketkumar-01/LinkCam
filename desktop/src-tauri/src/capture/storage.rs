use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppSettings {
    save_destination: String,
}

pub struct CaptureStorage {
    base_dir: RwLock<PathBuf>,
}

impl CaptureStorage {
    pub fn new() -> Self {
        let initial_dir = Self::load_persisted_dir().unwrap_or_else(Self::default_base_dir);
        let _ = fs::create_dir_all(&initial_dir);
        Self {
            base_dir: RwLock::new(initial_dir),
        }
    }

    fn default_base_dir() -> PathBuf {
        if let Ok(profile) = std::env::var("USERPROFILE") {
            PathBuf::from(profile).join("Pictures").join("LinkCam")
        } else {
            std::env::temp_dir().join("LinkCam")
        }
    }

    fn settings_file_path() -> Option<PathBuf> {
        let local_app_data = std::env::var("LOCALAPPDATA").ok()?;
        Some(PathBuf::from(local_app_data).join("LinkCam").join("settings.json"))
    }

    fn load_persisted_dir() -> Option<PathBuf> {
        let path = Self::settings_file_path()?;
        if path.exists() {
            let data = fs::read_to_string(&path).ok()?;
            let settings: AppSettings = serde_json::from_str(&data).ok()?;
            let p = PathBuf::from(settings.save_destination);
            if p.is_absolute() && p.exists() && p.is_dir() {
                return Some(p);
            }
        }
        None
    }

    fn persist_dir(dir: &Path) -> Result<(), String> {
        if let Some(path) = Self::settings_file_path() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let settings = AppSettings {
                save_destination: dir.to_string_lossy().to_string(),
            };
            let json = serde_json::to_string_pretty(&settings)
                .map_err(|e| format!("Serialization error: {}", e))?;
            fs::write(path, json).map_err(|e| format!("Failed to write settings: {}", e))?;
        }
        Ok(())
    }

    pub fn get_base_dir(&self) -> PathBuf {
        self.base_dir.read().unwrap().clone()
    }

    pub fn set_base_dir(&self, new_path_str: &str) -> Result<String, String> {
        let trimmed = new_path_str.trim();
        if trimmed.is_empty() {
            return Err("Destination path cannot be empty".to_string());
        }

        let path = PathBuf::from(trimmed);
        if !path.is_absolute() {
            return Err("Path must be an absolute Windows directory path (e.g. C:\\Users\\...)".to_string());
        }

        // Prohibit sensitive system root and OS directories
        let path_upper = trimmed.to_uppercase();
        if path_upper.starts_with("C:\\WINDOWS")
            || path_upper.starts_with("C:\\PROGRAM FILES")
            || path_upper.starts_with("C:\\PROGRAM FILES (X86)")
            || path_upper == "C:\\"
            || path_upper == "C:"
        {
            return Err("Access denied: Cannot set save destination to a protected system directory".to_string());
        }

        // Prevent path traversal sequences
        if trimmed.contains("..") {
            return Err("Path traversal sequences are not permitted".to_string());
        }

        // Create directory if it does not exist
        if !path.exists() {
            fs::create_dir_all(&path).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        if !path.is_dir() {
            return Err("Target path exists but is not a directory".to_string());
        }

        // Verify write access by touching a temporary file
        let test_file = path.join(".linkcam_write_test");
        match File::create(&test_file).and_then(|mut f| f.write_all(b"test")) {
            Ok(_) => {
                let _ = fs::remove_file(test_file);
            }
            Err(e) => {
                return Err(format!("Directory is not writable: {}", e));
            }
        }

        // Canonicalize path
        let canonical = path.canonicalize().unwrap_or(path);

        // Update state and persist
        {
            let mut guard = self.base_dir.write().unwrap();
            *guard = canonical.clone();
        }

        let _ = Self::persist_dir(&canonical);

        println!("[LinkCam Settings] Save destination updated to {:?}", canonical);
        Ok(canonical.to_string_lossy().to_string())
    }

    /// Save capture data to disk.
    /// CRITICAL SECURITY: The desktop engine generates the filename itself based on local timestamp
    /// and validated MIME type. Phone-supplied names are NEVER trusted.
    pub fn save_capture(&self, mime_type: &str, data: &[u8]) -> io::Result<PathBuf> {
        let ext = match mime_type {
            "application/pdf" => "pdf",
            "image/png" => "png",
            "image/webp" => "webp",
            _ => "jpg",
        };

        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let rand_id = &uuid::Uuid::new_v4().to_string()[..8];
        let desktop_file_name = format!("LinkCam_{}_{}.{}", timestamp, rand_id, ext);

        let base_dir = self.get_base_dir();
        let target_path = base_dir.join(&desktop_file_name);

        // Security assertion: target_path must be strictly inside base_dir
        if !target_path.starts_with(&base_dir) {
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
