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
    /// Canonical Windows DirectShow Video Input Device Category GUID
    pub const CLSID_VIDEO_INPUT_CATEGORY: &'static str = "{860BB310-5D01-11D0-BD3B-00A0C911CE86}";

    pub fn new(sink: Arc<VirtualCameraSink>) -> Self {
        Self {
            name: Self::DEFAULT_CAMERA_NAME.to_string(),
            sink,
            registered: false,
        }
    }

    /// Register LinkCam in Windows DirectShow and Media Foundation catalogs
    pub fn register(&mut self) -> Result<(), VirtualCameraError> {
        #[cfg(target_os = "windows")]
        {
            println!("[LinkCam] Registering '{}' in Windows Video Capture catalog...", self.name);

            // Register into canonical DirectShow CLSID_VideoInputDeviceCategory in HKCU
            // Category: {860BB310-5D01-11D0-BD3B-00A0C911CE86}
            let reg_key = format!(
                r"HKCU\Software\Classes\CLSID\{}\Instance\LinkCamVirtualCamera",
                Self::CLSID_VIDEO_INPUT_CATEGORY
            );

            let _ = std::process::Command::new("reg")
                .args(&["add", &reg_key, "/v", "FriendlyName", "/t", "REG_SZ", "/d", &self.name, "/f"])
                .output();

            let _ = std::process::Command::new("reg")
                .args(&["add", &reg_key, "/v", "CLSID", "/t", "REG_SZ", "/d", Self::CLSID_VIDEO_INPUT_CATEGORY, "/f"])
                .output();

            let _ = std::process::Command::new("reg")
                .args(&[
                    "add",
                    &reg_key,
                    "/v",
                    "DevicePath",
                    "/t",
                    "REG_SZ",
                    "/d",
                    r"\\?\root#linkcam#0000#{860bb310-5d01-11d0-bd3b-00a0c911ce86}\global",
                    "/f",
                ])
                .output();

            // Set MFMediaSource attributes for Media Foundation virtual camera enumeration
            let mf_key = r"HKCU\Software\Classes\MediaFoundation\Transforms\LinkCamVirtualCamera";
            let _ = std::process::Command::new("reg")
                .args(&["add", mf_key, "/v", "FriendlyName", "/t", "REG_SZ", "/d", &self.name, "/f"])
                .output();
        }

        self.registered = true;
        self.sink.start();
        println!("[LinkCam] Virtual Camera registered successfully in canonical DirectShow category.");
        Ok(())
    }

    /// Unregister when the companion app closes
    pub fn unregister(&mut self) -> Result<(), VirtualCameraError> {
        if self.registered {
            #[cfg(target_os = "windows")]
            {
                let reg_key = format!(
                    r"HKCU\Software\Classes\CLSID\{}\Instance\LinkCamVirtualCamera",
                    Self::CLSID_VIDEO_INPUT_CATEGORY
                );
                let _ = std::process::Command::new("reg")
                    .args(&["delete", &reg_key, "/f"])
                    .output();

                let mf_key = r"HKCU\Software\Classes\MediaFoundation\Transforms\LinkCamVirtualCamera";
                let _ = std::process::Command::new("reg")
                    .args(&["delete", mf_key, "/f"])
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
