use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdbDeviceInfo {
    pub serial: String,
    pub model: String,
    pub status: String,
    pub is_trusted: bool,
}

pub struct AdbManager;

impl AdbManager {
    /// Locate adb binary: checks system PATH, ANDROID_HOME, LocalAppData Android Sdk, and bundled paths
    pub fn locate_adb() -> String {
        // 1. Check system PATH first
        if let Ok(output) = Command::new("adb").arg("version").output() {
            if output.status.success() {
                return "adb".to_string();
            }
        }

        // 2. Check ANDROID_HOME / ANDROID_SDK_ROOT environment variables
        for env_var in ["ANDROID_HOME", "ANDROID_SDK_ROOT"] {
            if let Ok(root) = std::env::var(env_var) {
                let candidate = std::path::PathBuf::from(root).join("platform-tools").join("adb.exe");
                if candidate.exists() {
                    return candidate.to_string_lossy().to_string();
                }
            }
        }

        // 3. Check %LOCALAPPDATA%\Android\Sdk\platform-tools\adb.exe (Standard Android Studio installation on Windows)
        if let Ok(app_data) = std::env::var("LOCALAPPDATA") {
            let sdk_candidate = std::path::PathBuf::from(&app_data)
                .join("Android")
                .join("Sdk")
                .join("platform-tools")
                .join("adb.exe");
            if sdk_candidate.exists() {
                return sdk_candidate.to_string_lossy().to_string();
            }

            let linkcam_candidate = std::path::PathBuf::from(&app_data)
                .join("LinkCam")
                .join("platform-tools")
                .join("adb.exe");
            if linkcam_candidate.exists() {
                return linkcam_candidate.to_string_lossy().to_string();
            }
        }

        // 4. Common standalone installations
        for path in [
            r"C:\platform-tools\adb.exe",
            r"C:\Program Files (x86)\Android\android-sdk\platform-tools\adb.exe",
        ] {
            if std::path::Path::new(path).exists() {
                return path.to_string();
            }
        }

        "adb".to_string()
    }

    /// List connected adb devices
    pub fn list_devices() -> Result<Vec<AdbDeviceInfo>, String> {
        let adb = Self::locate_adb();
        let output = Command::new(&adb)
            .args(["devices", "-l"])
            .output()
            .map_err(|e| format!("Failed to execute adb: {}. Ensure adb is installed or phone is connected via USB.", e))?;

        if !output.status.success() {
            return Err("adb devices returned non-zero exit code".to_string());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut devices = Vec::new();

        for line in stdout.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let serial = parts[0].to_string();
                let status = parts[1].to_string();
                let model = parts.iter()
                    .find(|p| p.starts_with("model:"))
                    .map(|p| p.trim_start_matches("model:").replace('_', " "))
                    .unwrap_or_else(|| "Android Device".to_string());

                devices.push(AdbDeviceInfo {
                    serial,
                    model,
                    status,
                    is_trusted: false,
                });
            }
        }

        Ok(devices)
    }

    /// Setup reverse port forwarding strictly for TCP port 8443.
    /// CRITICAL ARCHITECTURAL NOTE: ADB reverse does NOT support UDP packets.
    /// Port 8444 (UDP datagram video) cannot be reversed over ADB.
    /// Over USB, all video, control, and telemetry flow through the TCP-framed connection on port 8443.
    pub fn setup_reverse(serial: Option<&str>, port: u16) -> Result<(), String> {
        let adb = Self::locate_adb();
        let mut cmd = Command::new(&adb);
        if let Some(s) = serial {
            if !s.is_empty() {
                cmd.args(["-s", s]);
            }
        }
        let forward_arg = format!("tcp:{}", port);
        cmd.args(["reverse", &forward_arg, &forward_arg]);

        let output = cmd.output().map_err(|e| format!("Failed to execute adb reverse: {}", e))?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("adb reverse failed: {}", err.trim()));
        }

        println!("[LinkCam ADB] Established reverse port forwarding for tcp:{}", port);
        Ok(())
    }

    /// Push one-time ephemeral USB token and certificate fingerprint to the phone app via Android Intent
    pub fn push_pairing_token(
        serial: &str,
        token: &str,
        session_id: &str,
        cert_fp: &str,
        port: u16,
    ) -> Result<(), String> {
        let adb = Self::locate_adb();
        let mut cmd = Command::new(&adb);
        if !serial.is_empty() {
            cmd.args(["-s", serial]);
        }
        cmd.args([
            "shell", "am", "start",
            "-n", "com.linkcam.app/.ui.MainActivity",
            "--es", "usb_token", token,
            "--es", "session_id", session_id,
            "--es", "cert_fp", cert_fp,
            "--ei", "port", &port.to_string(),
        ]);

        let output = cmd.output().map_err(|e| format!("Failed to push token via adb intent: {}", e))?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("adb push intent failed: {}", err.trim()));
        }

        println!("[LinkCam ADB] Pushed one-time USB pairing token and TLS fingerprint to device {}", serial);
        Ok(())
    }
}
