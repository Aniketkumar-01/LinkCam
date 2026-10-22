use super::frame_sink::VirtualCameraSink;
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
    #[cfg(target_os = "windows")]
    mf_vcam: Option<windows::Win32::Media::MediaFoundation::IMFVirtualCamera>,
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
            #[cfg(target_os = "windows")]
            mf_vcam: None,
        }
    }

    fn find_directshow_dll() -> Option<PathBuf> {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));

        let mut candidates = Vec::new();

        if let Some(ref dir) = exe_dir {
            candidates.push(dir.join("LinkCamDShow64.dll"));
            candidates.push(dir.join("LinkCamDShow.dll"));
            candidates.push(dir.join(r"vcam-dshow\LinkCamDShow64.dll"));
            candidates.push(dir.join(r"..\vcam-dshow\LinkCamDShow64.dll"));
            candidates.push(dir.join(r"..\..\vcam-dshow\LinkCamDShow64.dll"));
            candidates.push(dir.join(r"..\..\..\vcam-dshow\LinkCamDShow64.dll"));
        }

        if let Ok(cwd) = std::env::current_dir() {
            candidates.push(cwd.join("LinkCamDShow64.dll"));
            candidates.push(cwd.join(r"desktop\vcam-dshow\LinkCamDShow64.dll"));
            candidates.push(cwd.join(r"vcam-dshow\LinkCamDShow64.dll"));
            candidates.push(cwd.join(r"..\vcam-dshow\LinkCamDShow64.dll"));
        }

        candidates.push(PathBuf::from(r"d:\AG\LinkCam\desktop\vcam-dshow\LinkCamDShow64.dll"));
        candidates.push(PathBuf::from(r"D:\Programs\LinkCam\LinkCamDShow64.dll"));
        candidates.push(PathBuf::from(r"C:\Program Files\LinkCam\LinkCamDShow64.dll"));

        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            candidates.push(PathBuf::from(local_app_data).join(r"Programs\LinkCam\LinkCamDShow64.dll"));
        }
        if let Ok(prog_files) = std::env::var("ProgramFiles") {
            candidates.push(PathBuf::from(prog_files).join(r"LinkCam\LinkCamDShow64.dll"));
        }

        candidates.into_iter().find(|p| p.exists())
    }

    /// Purge legacy and duplicate registry keys from earlier LinkCam builds
    fn clean_legacy_registries() {
        let keys_to_delete = [
            r"HKCU\Software\Classes\CLSID\{860BB310-5D01-11D0-BD3B-00A0C911CE86}\Instance\LinkCamVirtualCamera",
            r"HKCU\Software\Classes\CLSID\{860BB310-5D01-11D0-BD3B-00A0C911CE86}\Instance\LinkCam Virtual Camera",
            r"HKCU\Software\Classes\CLSID\{86497AEE-B071-11D0-A3A0-00A0C9223196}\Instance\LinkCamVirtualCamera",
            r"HKCU\Software\Classes\CLSID\{86497AEE-B071-11D0-A3A0-00A0C9223196}\Instance\LinkCam Virtual Camera",
            r"HKCU\Software\Classes\CLSID\{86497AEE-B071-11D0-A3A0-00A0C9223196}\Instance\{D37E5526-92F7-4E61-A76F-B590E8F88E29}",
            r"HKLM\Software\Classes\CLSID\{860BB310-5D01-11D0-BD3B-00A0C911CE86}\Instance\LinkCamVirtualCamera",
            r"HKLM\Software\Classes\CLSID\{860BB310-5D01-11D0-BD3B-00A0C911CE86}\Instance\LinkCam Virtual Camera",
            r"HKLM\Software\Classes\CLSID\{86497AEE-B071-11D0-A3A0-00A0C9223196}\Instance\LinkCamVirtualCamera",
            r"HKLM\Software\Classes\CLSID\{86497AEE-B071-11D0-A3A0-00A0C9223196}\Instance\LinkCam Virtual Camera",
        ];

        for key in keys_to_delete {
            let _ = std::process::Command::new("reg.exe")
                .args(&["delete", key, "/f"])
                .output();
        }
    }

    /// Ensure DevicePath, InprocServer32, and FilterData are written for OBS Studio libdshowcapture and DirectShow recognition
    fn ensure_device_path(dll_path: Option<&PathBuf>) {
        let key = r"HKCU\Software\Classes\CLSID\{860BB310-5D01-11D0-BD3B-00A0C911CE86}\Instance\{D37E5526-92F7-4E61-A76F-B590E8F88E29}";
        let _ = std::process::Command::new("reg.exe")
            .args(&["add", key, "/v", "DevicePath", "/t", "REG_SZ", "/d", r"\\?\root#media#linkcam_virtual_camera", "/f"])
            .output();
        let _ = std::process::Command::new("reg.exe")
            .args(&["add", key, "/v", "FriendlyName", "/t", "REG_SZ", "/d", "LinkCam Virtual Camera", "/f"])
            .output();
        let _ = std::process::Command::new("reg.exe")
            .args(&["add", key, "/v", "CLSID", "/t", "REG_SZ", "/d", "{D37E5526-92F7-4E61-A76F-B590E8F88E29}", "/f"])
            .output();
        let _ = std::process::Command::new("reg.exe")
            .args(&["add", key, "/v", "Description", "/t", "REG_SZ", "/d", "LinkCam DirectShow Virtual Webcam", "/f"])
            .output();

        // If HKLM already has the DirectShow FilterData binary blob, copy it into HKCU so user-mode enumerators discover it immediately
        let hklm_key = r"HKLM\Software\Classes\CLSID\{860BB310-5D01-11D0-BD3B-00A0C911CE86}\Instance\{D37E5526-92F7-4E61-A76F-B590E8F88E29}";
        let _ = std::process::Command::new("reg.exe")
            .args(&["copy", hklm_key, key, "/s", "/f"])
            .output();

        if let Some(dll) = dll_path {
            let clsid_key = r"HKCU\Software\Classes\CLSID\{D37E5526-92F7-4E61-A76F-B590E8F88E29}";
            let _ = std::process::Command::new("reg.exe")
                .args(&["add", clsid_key, "/ve", "/t", "REG_SZ", "/d", "LinkCam Virtual Camera", "/f"])
                .output();
            let inproc_key = format!(r"{}\InprocServer32", clsid_key);
            let _ = std::process::Command::new("reg.exe")
                .args(&["add", &inproc_key, "/ve", "/t", "REG_SZ", "/d", &dll.to_string_lossy(), "/f"])
                .output();
            let _ = std::process::Command::new("reg.exe")
                .args(&["add", &inproc_key, "/v", "ThreadingModel", "/t", "REG_SZ", "/d", "Both", "/f"])
                .output();
        }
    }

    /// Register LinkCam in Windows DirectShow (Win 10/11) and Media Foundation (Win 11)
    pub fn register(&mut self) -> Result<(), VirtualCameraError> {
        self.sink.start();

        #[cfg(target_os = "windows")]
        {
            // Re-probe for the filter DLL in case it was compiled after launch
            if self.filter_dll_path.is_none() {
                self.filter_dll_path = Self::find_directshow_dll();
            }

            // Clean any stale / duplicate entries first so only one entry appears
            Self::clean_legacy_registries();

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
                    "[LinkCam] ATTENTION: LinkCamDShow64.dll not found in search paths. Run 'desktop\\vcam-dshow\\build.bat' to build and register the DirectShow filter DLL."
                );
            }

            // Ensure DevicePath, InprocServer32, and FilterData are explicitly written for OBS Studio and Zoom
            Self::ensure_device_path(self.filter_dll_path.as_ref());

            // 2. Windows 11 Media Foundation Virtual Camera API registration & activation
            self.mf_vcam = Self::try_register_windows11_mf(&self.name, Self::CLSID_LINKCAM_DSHOW_FILTER);
            if self.mf_vcam.is_some() {
                println!("[LinkCam] Windows 11 Media Foundation Virtual Camera registration active and started.");
            } else {
                println!("[LinkCam] Media Foundation Virtual Camera fallback: DirectShow COM filter is active as primary virtual camera path.");
            }
        }

        self.registered = true;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn try_register_windows11_mf(friendly_name: &str, clsid_str: &str) -> Option<windows::Win32::Media::MediaFoundation::IMFVirtualCamera> {
        use windows::core::w;
        use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
        use windows::Win32::Media::MediaFoundation::IMFVirtualCamera;

        type FnMFCreateVirtualCamera = unsafe extern "system" fn(
            camera_type: u32,
            lifetime: u32,
            access: u32,
            friendly_name: *const u16,
            source_id: *const u16,
            categories: *const windows::core::GUID,
            category_count: u32,
            virtual_camera: *mut *mut std::ffi::c_void,
        ) -> windows::core::HRESULT;

        unsafe {
            let h_mod = match LoadLibraryW(w!("mfplat.dll")) {
                Ok(h) => h,
                Err(_) => return None,
            };

            let proc = GetProcAddress(h_mod, windows::core::s!("MFCreateVirtualCamera"));
            let mf_create: FnMFCreateVirtualCamera = match proc {
                Some(p) => std::mem::transmute(p),
                None => return None,
            };

            let name_wide: Vec<u16> = friendly_name.encode_utf16().chain(std::iter::once(0)).collect();
            let id_wide: Vec<u16> = clsid_str.encode_utf16().chain(std::iter::once(0)).collect();

            let mut vcam_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let hr = mf_create(
                0, // MFVirtualCameraType_SoftwareCameraSource
                0, // MFVirtualCameraLifetime_Session
                0, // MFVirtualCameraAccess_CurrentUser
                name_wide.as_ptr(),
                id_wide.as_ptr(),
                std::ptr::null(),
                0,
                &mut vcam_ptr,
            );

            if hr.is_ok() && !vcam_ptr.is_null() {
                let vcam: IMFVirtualCamera = std::mem::transmute(vcam_ptr);
                let start_res = vcam.Start(None);
                if start_res.is_ok() {
                    println!("[LinkCam] Windows 11 user-mode Virtual Camera successfully created and STARTED via MFCreateVirtualCamera!");
                } else {
                    println!("[LinkCam] Windows 11 Virtual Camera created, Start returned: {:?}", start_res);
                }
                Some(vcam)
            } else {
                None
            }
        }
    }

    /// Unregister when the companion app closes
    pub fn unregister(&mut self) -> Result<(), VirtualCameraError> {
        if self.registered {
            #[cfg(target_os = "windows")]
            {
                if let Some(vcam) = self.mf_vcam.take() {
                    unsafe {
                        let _ = vcam.Stop();
                    }
                    println!("[LinkCam] Media Foundation Virtual Camera stopped.");
                }

                if let Some(dll_path) = &self.filter_dll_path {
                    let dll_str = dll_path.to_string_lossy();
                    let _ = std::process::Command::new("regsvr32.exe")
                        .args(&["/u", "/s", &dll_str])
                        .output();
                    println!("[LinkCam] DirectShow COM filter DLL unregistered.");
                }
                Self::clean_legacy_registries();
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

