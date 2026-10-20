use super::frame_sink::{VirtualCameraSink, SHM_EVENT_NAME, SHM_NAME};
use std::path::PathBuf;
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
    filter_dll_path: Option<PathBuf>,
}

impl VirtualCameraDevice {
    pub const DEFAULT_CAMERA_NAME: &'static str = "LinkCam Virtual Camera";
    /// Canonical Windows DirectShow Video Input Device Category GUID
    pub const CLSID_VIDEO_INPUT_CATEGORY: &'static str = "{860BB310-5D01-11D0-BD3B-00A0C911CE86}";
    /// Canonical LinkCam DirectShow Filter CLSID
    pub const CLSID_LINKCAM_DSHOW_FILTER: &'static str = "{D37E5526-92F7-4E61-A76F-B590E8F88E29}";

    pub fn new(sink: Arc<VirtualCameraSink>) -> Self {
        let filter_dll_path = Self::find_directshow_dll();
        Self {
            name: Self::DEFAULT_CAMERA_NAME.to_string(),
            sink,
            registered: false,
            filter_dll_path,
        }
    }

    fn find_directshow_dll() -> Option<PathBuf> {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))?;

        let candidates = [
            exe_dir.join("LinkCamDShow64.dll"),
            exe_dir.join("LinkCamDShow.dll"),
            PathBuf::from(r"C:\Program Files\LinkCam\LinkCamDShow64.dll"),
        ];

        candidates.into_iter().find(|p| p.exists())
    }

    /// Register LinkCam in Windows DirectShow (Win 10/11) and Media Foundation (Win 11)
    pub fn register(&mut self) -> Result<(), VirtualCameraError> {
        self.sink.start();

        #[cfg(target_os = "windows")]
        {
            // 1. DirectShow COM Filter Registration (Universal Win 10 & Win 11 compatibility)
            if let Some(dll_path) = &self.filter_dll_path {
                let dll_str = dll_path.to_string_lossy();
                println!("[LinkCam] Registering DirectShow COM filter DLL: {}", dll_str);

                let status = std::process::Command::new("regsvr32.exe")
                    .args(&["/s", &dll_str])
                    .status();

                match status {
                    Ok(s) if s.success() => {
                        println!("[LinkCam] Successfully registered DirectShow filter DLL with Windows.");
                    }
                    Ok(s) => {
                        eprintln!("[LinkCam] Warning: regsvr32 exited with status {:?}", s.code());
                    }
                    Err(e) => {
                        eprintln!("[LinkCam] Warning: Failed to run regsvr32: {}", e);
                    }
                }
            } else {
                println!(
                    "[LinkCam] Shared Memory '{}' and event '{}' are active for video streaming. (DirectShow Filter DLL not found in executable directory; install via installer or run regsvr32 LinkCamDShow64.dll to expose camera to Windows apps).",
                    SHM_NAME, SHM_EVENT_NAME
                );
            }

            // 2. Windows 11 Media Foundation Virtual Camera API probe
            let is_win11_mf = Self::check_windows11_mf();
            if is_win11_mf {
                println!("[LinkCam] Windows 11 Media Foundation Virtual Camera API verified (mfplat!MFCreateVirtualCamera available).");
            } else {
                println!("[LinkCam] Windows 10 environment detected: DirectShow COM filter is active as the primary virtual camera path.");
            }
        }

        self.registered = true;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn check_windows11_mf() -> bool {
        use windows::core::w;
        use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

        unsafe {
            let h_mod = LoadLibraryW(w!("mfplat.dll"));
            if let Ok(handle) = h_mod {
                let proc = GetProcAddress(handle, windows::core::s!("MFCreateVirtualCamera"));
                return proc.is_some();
            }
        }
        false
    }

    /// Unregister when the companion app closes
    pub fn unregister(&mut self) -> Result<(), VirtualCameraError> {
        if self.registered {
            #[cfg(target_os = "windows")]
            {
                if let Some(dll_path) = &self.filter_dll_path {
                    let dll_str = dll_path.to_string_lossy();
                    let _ = std::process::Command::new("regsvr32.exe")
                        .args(&["/u", "/s", &dll_str])
                        .output();
                    println!("[LinkCam] DirectShow COM filter DLL unregistered.");
                }
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

