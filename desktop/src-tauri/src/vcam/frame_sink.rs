use linkcam_protocol::Packet;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

pub const SHM_MAGIC: u32 = 0x4C4E4B43; // 'LNKC'
pub const SHM_VERSION: u32 = 1;
pub const SHM_NAME: &str = "Local\\LinkCam_Video_Shm";
pub const SHM_EVENT_NAME: &str = "Local\\LinkCam_Video_Event";
pub const SHM_CAPACITY_BYTES: usize = 32 * 1024 * 1024; // 32 MB buffer (supports 4K NV12 12.44MB frames)

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ShmVideoHeader {
    pub magic: u32,
    pub version: u32,
    pub width: u32,
    pub height: u32,
    pub format: u32, // 0 = H264 NAL, 1 = NV12, 2 = RGB32
    pub sequence: u32,
    pub timestamp_us: u64,
    pub payload_size: u32,
    pub is_keyframe: u32,
    pub reserved: [u32; 6],
}

pub const SHM_HEADER_SIZE: usize = std::mem::size_of::<ShmVideoHeader>();

#[derive(Debug, Clone)]
pub struct VideoFrameInfo {
    pub width: u32,
    pub height: u32,
    pub timestamp_us: u64,
    pub sequence: u32,
    pub is_keyframe: bool,
    pub data: Vec<u8>,
}

#[cfg(target_os = "windows")]
struct WinShmState {
    file_mapping: windows::Win32::Foundation::HANDLE,
    mapped_ptr: *mut u8,
    event_handle: windows::Win32::Foundation::HANDLE,
}

#[cfg(target_os = "windows")]
unsafe impl Send for WinShmState {}
#[cfg(target_os = "windows")]
unsafe impl Sync for WinShmState {}

#[cfg(target_os = "windows")]
impl Drop for WinShmState {
    fn drop(&mut self) {
        unsafe {
            if !self.mapped_ptr.is_null() {
                let _ = windows::Win32::System::Memory::UnmapViewOfFile(
                    windows::Win32::System::Memory::MEMORY_MAPPED_VIEW_ADDRESS {
                        Value: self.mapped_ptr as *mut _,
                    },
                );
            }
            if !self.file_mapping.is_invalid() {
                let _ = windows::Win32::Foundation::CloseHandle(self.file_mapping);
            }
            if !self.event_handle.is_invalid() {
                let _ = windows::Win32::Foundation::CloseHandle(self.event_handle);
            }
        }
    }
}

/// Windows Virtual Camera Video Frame Sink
pub struct VirtualCameraSink {
    is_active: AtomicBool,
    frames_processed: AtomicU64,
    latest_frame: Arc<RwLock<Option<VideoFrameInfo>>>,
    #[cfg(target_os = "windows")]
    win_shm: Option<WinShmState>,
}

impl VirtualCameraSink {
    pub fn new() -> Self {
        #[cfg(target_os = "windows")]
        let win_shm = Self::init_windows_shm();

        Self {
            is_active: AtomicBool::new(false),
            frames_processed: AtomicU64::new(0),
            latest_frame: Arc::new(RwLock::new(None)),
            #[cfg(target_os = "windows")]
            win_shm,
        }
    }

    #[cfg(target_os = "windows")]
    fn init_windows_shm() -> Option<WinShmState> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows::core::{w, PCWSTR};
        use windows::Win32::Foundation::{INVALID_HANDLE_VALUE, LocalFree, HLOCAL};
        use windows::Win32::Security::{SECURITY_ATTRIBUTES, PSECURITY_DESCRIPTOR};
        use windows::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
        use windows::Win32::System::Memory::{
            CreateFileMappingW, MapViewOfFile, FILE_MAP_ALL_ACCESS, PAGE_READWRITE,
        };
        use windows::Win32::System::Threading::CreateEventW;

        let wide_name: Vec<u16> = OsStr::new(SHM_NAME).encode_wide().chain(Some(0)).collect();
        let wide_event: Vec<u16> = OsStr::new(SHM_EVENT_NAME).encode_wide().chain(Some(0)).collect();

        // SDDL:
        // D:(A;;GA;;;OW) - Full control to current owner
        // (A;;GR;;;WD)   - Read-only to Everyone (World)
        // (A;;GR;;;AC)   - Read-only to ALL APPLICATION PACKAGES (AppContainer / UWP / Store Apps)
        // S:(ML;;NW;;;LW) - Mandatory Label: No-Write-Up for Low Mandatory Level
        let sddl = w!("D:(A;;GA;;;OW)(A;;GR;;;WD)(A;;GR;;;AC)S:(ML;;NW;;;LW)");
        let mut sec_desc = PSECURITY_DESCRIPTOR::default();
        let mut sec_attr = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: std::ptr::null_mut(),
            bInheritHandle: false.into(),
        };

        let has_sddl = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl,
                1, // SDDL_REVISION_1
                &mut sec_desc,
                None,
            ).is_ok()
        };

        if has_sddl && !sec_desc.0.is_null() {
            sec_attr.lpSecurityDescriptor = sec_desc.0;
        }

        let p_sec_attr = if !sec_attr.lpSecurityDescriptor.is_null() {
            Some(&sec_attr as *const _)
        } else {
            None
        };

        unsafe {
            let file_mapping = CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                if let Some(sa) = p_sec_attr { Some(&*sa) } else { None },
                PAGE_READWRITE,
                0,
                SHM_CAPACITY_BYTES as u32,
                PCWSTR(wide_name.as_ptr()),
            ).ok();

            let file_mapping = match file_mapping {
                Some(h) => h,
                None => {
                    if !sec_desc.0.is_null() {
                        let _ = LocalFree(HLOCAL(sec_desc.0));
                    }
                    return None;
                }
            };

            let view = MapViewOfFile(
                file_mapping,
                FILE_MAP_ALL_ACCESS,
                0,
                0,
                SHM_CAPACITY_BYTES,
            );

            let mapped_ptr = view.Value as *mut u8;
            if mapped_ptr.is_null() {
                let _ = windows::Win32::Foundation::CloseHandle(file_mapping);
                if !sec_desc.0.is_null() {
                    let _ = LocalFree(HLOCAL(sec_desc.0));
                }
                return None;
            }

            // Create MANUAL-RESET event so multiple consumers (e.g. Zoom and OBS running concurrently)
            // all wake up without stealing frames from each other.
            let event_handle = CreateEventW(
                if let Some(sa) = p_sec_attr { Some(&*sa) } else { None },
                true,  // Manual-reset
                false, // Initial state unsignaled
                PCWSTR(wide_event.as_ptr()),
            ).ok();

            if !sec_desc.0.is_null() {
                let _ = LocalFree(HLOCAL(sec_desc.0));
            }

            let event_handle = match event_handle {
                Some(h) => h,
                None => {
                    let _ = windows::Win32::System::Memory::UnmapViewOfFile(
                        windows::Win32::System::Memory::MEMORY_MAPPED_VIEW_ADDRESS {
                            Value: mapped_ptr as *mut _,
                        },
                    );
                    let _ = windows::Win32::Foundation::CloseHandle(file_mapping);
                    return None;
                }
            };

            println!(
                "[LinkCam] Virtual Camera Shared Memory initialized at '{}' (32 MB, Low-Integrity & AppContainer SDDL active)",
                SHM_NAME
            );
            Some(WinShmState {
                file_mapping,
                mapped_ptr,
                event_handle,
            })
        }
    }

    pub fn start(&self) {
        self.is_active.store(true, Ordering::SeqCst);
    }

    pub fn stop(&self) {
        self.is_active.store(false, Ordering::SeqCst);
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    pub async fn push_packet(&self, packet: Packet) {
        if !self.is_active() {
            return;
        }

        let payload_len = packet.payload.len();

        #[cfg(target_os = "windows")]
        if let Some(shm) = &self.win_shm {
            if !shm.mapped_ptr.is_null() {
                let max_payload = SHM_CAPACITY_BYTES - SHM_HEADER_SIZE;
                let copy_len = payload_len.min(max_payload);

                let base_seq = packet.header.sequence.max(1) * 2;
                let header = ShmVideoHeader {
                    magic: SHM_MAGIC,
                    version: SHM_VERSION,
                    width: 1920,
                    height: 1080,
                    format: 0, // H264
                    sequence: base_seq - 1, // Odd = write in progress (seqlock guard)
                    timestamp_us: packet.header.timestamp,
                    payload_size: copy_len as u32,
                    is_keyframe: if packet.header.is_keyframe() { 1 } else { 0 },
                    reserved: [0; 6],
                };

                unsafe {
                    let _ = windows::Win32::System::Threading::ResetEvent(shm.event_handle);
                    // Write header with odd sequence
                    std::ptr::copy_nonoverlapping(
                        &header as *const _ as *const u8,
                        shm.mapped_ptr,
                        SHM_HEADER_SIZE,
                    );
                    // Copy payload
                    std::ptr::copy_nonoverlapping(
                        packet.payload.as_ptr(),
                        shm.mapped_ptr.add(SHM_HEADER_SIZE),
                        copy_len,
                    );
                    // Commit sequence = base_seq (even = write complete)
                    let seq_ptr = shm.mapped_ptr.add(20) as *mut u32;
                    std::ptr::write_volatile(seq_ptr, base_seq);

                    let _ = windows::Win32::System::Threading::SetEvent(shm.event_handle);
                }
            }
        }

        let frame_info = VideoFrameInfo {
            width: 1920,
            height: 1080,
            timestamp_us: packet.header.timestamp,
            sequence: packet.header.sequence,
            is_keyframe: packet.header.is_keyframe(),
            data: packet.payload,
        };

        {
            let mut latest = self.latest_frame.write().await;
            *latest = Some(frame_info);
        }

        self.frames_processed.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn get_latest_frame(&self) -> Option<VideoFrameInfo> {
        let latest = self.latest_frame.read().await;
        latest.clone()
    }

    pub fn total_frames(&self) -> u64 {
        self.frames_processed.load(Ordering::Relaxed)
    }
}

