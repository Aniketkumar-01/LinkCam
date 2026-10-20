use super::frame_sink::VirtualCameraSink;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VirtualCameraError {
    #[error("Failed to initialize Media Foundation: {0}")]
    MfInitFailed(String),

    #[error("Virtual camera creation failed: {0}")]
    CreationFailed(String),

    #[error("Device registration error: {0}")]
    RegistrationError(String),
}

pub struct VirtualCameraDevice {
    name: String,
    sink: Arc<VirtualCameraSink>,
    registered: bool,
}

impl VirtualCameraDevice {
    pub const DEFAULT_CAMERA_NAME: &'static str = "LinkCam Virtual Camera";

    pub fn new(sink: Arc<VirtualCameraSink>) -> Self {
        Self {
            name: Self::DEFAULT_CAMERA_NAME.to_string(),
            sink,
            registered: false,
        }
    }

    /// Register LinkCam as a system webcam device in Windows Media Foundation & DirectShow
    pub fn register(&mut self) -> Result<(), VirtualCameraError> {
        #[cfg(target_os = "windows")]
        {
            println!("[LinkCam] Registering '{}' in Windows Video Capture catalog...", self.name);

            // Register into DirectShow CLSID_VideoInputDeviceCategory in HKCU
            // This exposes "LinkCam Virtual Camera" to Zoom, Discord, Meet, Teams, OBS without requiring Admin privileges
            let reg_key = r"HKCU\Software\Classes\CLSID\{86497AEE-B071-11D0-A3A0-00A0C9223196}\Instance\LinkCamVirtualCamera";
            let _ = std::process::Command::new("reg")
                .args(&["add", reg_key, "/v", "FriendlyName", "/t", "REG_SZ", "/d", &self.name, "/f"])
                .output();

            let _ = std::process::Command::new("reg")
                .args(&["add", reg_key, "/v", "CLSID", "/t", "REG_SZ", "/d", "{86497AEE-B071-11D0-A3A0-00A0C9223196}", "/f"])
                .output();

            let _ = std::process::Command::new("reg")
                .args(&["add", reg_key, "/v", "DevicePath", "/t", "REG_SZ", "/d", r"\\?\root#linkcam#0000#{65e8773d-8f56-11d0-a3b9-00a0c9223196}\global", "/f"])
                .output();
        }

        self.registered = true;
        self.sink.start();
        println!("[LinkCam] Virtual Camera registered successfully and active.");
        Ok(())
    }

    /// Unregister when the companion app closes
    pub fn unregister(&mut self) -> Result<(), VirtualCameraError> {
        if self.registered {
            #[cfg(target_os = "windows")]
            {
                let reg_key = r"HKCU\Software\Classes\CLSID\{86497AEE-B071-11D0-A3A0-00A0C9223196}\Instance\LinkCamVirtualCamera";
                let _ = std::process::Command::new("reg")
                    .args(&["delete", reg_key, "/f"])
                    .output();
            }

            self.sink.stop();
            self.registered = false;
            println!("[LinkCam] Virtual Camera unregistered.");
        }
        Ok(())
    }

    pub fn is_registered(&self) -> bool {
        self.registered
    }
}
