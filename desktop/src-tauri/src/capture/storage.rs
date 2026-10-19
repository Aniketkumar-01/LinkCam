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
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let safe_name = format!("{}_{}", timestamp, file_name);
        let target_path = self.base_dir.join(safe_name);

        let mut file = File::create(&target_path)?;
        file.write_all(data)?;
        println!("[LinkCam] Saved instant capture to {:?}", target_path);

        Ok(target_path)
    }
}

fn dirs_base() -> PathBuf {
    if let Ok(profile) = std::env::var("USERPROFILE") {
        PathBuf::from(profile).join("Pictures")
    } else {
        std::env::temp_dir()
    }
}
