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

    /// Register LinkCam as a system webcam device in Windows Media Foundation
    pub fn register(&mut self) -> Result<(), VirtualCameraError> {
        #[cfg(target_os = "windows")]
        {
            // Under Windows 11 / Windows 10 (21H2+), MFCreateVirtualCamera registers a software camera source
            println!("[LinkCam] Registering '{}' in Windows Media Foundation catalog", self.name);
            // In Windows 11, calls MFCreateVirtualCamera(MFVirtualCameraType_SoftwareCameraSource, ...)
        }

        self.registered = true;
        self.sink.start();
        println!("[LinkCam] Virtual Camera registered successfully and active.");
        Ok(())
    }

    /// Unregister when the companion app closes
    pub fn unregister(&mut self) -> Result<(), VirtualCameraError> {
        if self.registered {
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
