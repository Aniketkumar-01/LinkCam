//! Shared memory frame ring for LinkCam Virtual Camera (Media Foundation & DirectShow).
//! Direct port of Owlmic's proven 2-slot seqlock ring buffer.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

pub const NAME: &str = "Global\\LinkCamCamFrames";
pub const WIDTH: usize = 1920;
pub const HEIGHT: usize = 1080;
pub const FRAME_BYTES: usize = WIDTH * HEIGHT * 3 / 2;
const MAGIC: u32 = u32::from_le_bytes(*b"LKCM");
const HEADER: usize = 64;
pub const TOTAL: usize = HEADER + 2 * FRAME_BYTES;
/// If no new frame arrives for 2 seconds, the camera gracefully serves the placeholder.
pub const STALE_MS: u64 = 2_000;
const NONE: u32 = u32::MAX;

// Header offsets
const OFF_MAGIC: usize = 0;
const OFF_LATEST: usize = 4;
const OFF_SEQ: usize = 8; // two u32, one per slot
const OFF_FRAMES: usize = 16;
const OFF_WRITTEN_MS: usize = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Read {
    Picture,
    /// Writer lapped the reader mid-copy; reader keeps previous picture
    Torn,
    /// No picture, or frame is older than STALE_MS
    Nothing,
}

/// A view of the shared memory frame ring.
pub struct Ring {
    base: *mut u8,
}

unsafe impl Send for Ring {}
unsafe impl Sync for Ring {}

impl Ring {
    /// # Safety
    /// `base` must point to at least `TOTAL` bytes, 8-byte aligned, valid while Ring exists.
    pub unsafe fn new(base: *mut u8) -> Self {
        Self { base }
    }

    fn u32_at(&self, off: usize) -> &AtomicU32 {
        unsafe { AtomicU32::from_ptr(self.base.add(off) as *mut u32) }
    }

    fn u64_at(&self, off: usize) -> &AtomicU64 {
        unsafe { AtomicU64::from_ptr(self.base.add(off) as *mut u64) }
    }

    fn slot(&self, i: usize) -> *mut u8 {
        unsafe { self.base.add(HEADER + i * FRAME_BYTES) }
    }

    /// Initializes a fresh ring header.
    pub fn init(&self) {
        self.u32_at(OFF_LATEST).store(NONE, Ordering::Release);
        self.u32_at(OFF_SEQ).store(0, Ordering::Release);
        self.u32_at(OFF_SEQ + 4).store(0, Ordering::Release);
        self.u64_at(OFF_FRAMES).store(0, Ordering::Release);
        self.u64_at(OFF_WRITTEN_MS).store(0, Ordering::Release);
        self.u32_at(OFF_MAGIC).store(MAGIC, Ordering::Release);
    }

    pub fn is_ready(&self) -> bool {
        self.u32_at(OFF_MAGIC).load(Ordering::Acquire) == MAGIC
    }

    /// Publishes one 1920x1080 NV12 picture into the inactive slot.
    pub fn write(&self, frame: &[u8], now_ms: u64) {
        if frame.len() != FRAME_BYTES {
            return;
        }
        let latest = self.u32_at(OFF_LATEST).load(Ordering::Acquire);
        let slot = if latest == 0 { 1 } else { 0 };
        let seq = self.u32_at(OFF_SEQ + slot * 4);
        seq.fetch_add(1, Ordering::AcqRel); // Odd = writing
        unsafe {
            std::ptr::copy_nonoverlapping(frame.as_ptr(), self.slot(slot), FRAME_BYTES);
        }
        seq.fetch_add(1, Ordering::AcqRel); // Even = complete
        self.u32_at(OFF_LATEST).store(slot as u32, Ordering::Release);
        self.u64_at(OFF_FRAMES).fetch_add(1, Ordering::AcqRel);
        self.u64_at(OFF_WRITTEN_MS).store(now_ms, Ordering::Release);
    }

    /// Copies the newest picture into `out` (must be at least FRAME_BYTES).
    pub fn read(&self, out: &mut [u8], now_ms: u64) -> Read {
        let latest = self.u32_at(OFF_LATEST).load(Ordering::Acquire);
        if latest == NONE
            || now_ms.saturating_sub(self.u64_at(OFF_WRITTEN_MS).load(Ordering::Acquire)) > STALE_MS
        {
            return Read::Nothing;
        }
        let seq = self.u32_at(OFF_SEQ + latest as usize * 4);
        let before = seq.load(Ordering::Acquire);
        if before % 2 == 1 {
            return Read::Torn;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(self.slot(latest as usize), out.as_mut_ptr(), FRAME_BYTES);
        }
        std::sync::atomic::fence(Ordering::Acquire);
        if seq.load(Ordering::Relaxed) == before {
            Read::Picture
        } else {
            Read::Torn
        }
    }

    pub fn frames_written(&self) -> u64 {
        self.u64_at(OFF_FRAMES).load(Ordering::Acquire)
    }
}

/// Helper that serves the latest picture, retains the previous on torn reads, or falls back to placeholder.
pub struct Picker {
    scratch: Vec<u8>,
    shown: Vec<u8>,
    placeholder: Vec<u8>,
    has_picture: bool,
}

impl Picker {
    pub fn new(placeholder: Vec<u8>) -> Self {
        assert_eq!(placeholder.len(), FRAME_BYTES);
        Self {
            scratch: vec![0; FRAME_BYTES],
            shown: vec![0; FRAME_BYTES],
            placeholder,
            has_picture: false,
        }
    }

    pub fn next(&mut self, ring: Option<&Ring>, now_ms: u64) -> &[u8] {
        match ring.map_or(Read::Nothing, |r| r.read(&mut self.scratch, now_ms)) {
            Read::Picture => {
                std::mem::swap(&mut self.scratch, &mut self.shown);
                self.has_picture = true;
            }
            Read::Torn => {}
            Read::Nothing => self.has_picture = false,
        }
        if self.has_picture {
            &self.shown
        } else {
            &self.placeholder
        }
    }
}

#[cfg(windows)]
pub use mapping::Mapping;

#[cfg(windows)]
mod mapping {
    use super::{NAME, Ring, TOTAL};
    use windows::Win32::Foundation::{
        CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HLOCAL, INVALID_HANDLE_VALUE,
        LocalFree,
    };
    use windows::Win32::Security::Authorization::{
        ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
    };
    use windows::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES};
    use windows::Win32::System::Memory::{
        CreateFileMappingW, FILE_MAP_ALL_ACCESS, MEMORY_MAPPED_VIEW_ADDRESS, MapViewOfFile,
        OpenFileMappingW, PAGE_READWRITE, UnmapViewOfFile,
    };
    use windows::core::{HSTRING, w};

    pub struct Mapping {
        handle: HANDLE,
        view: MEMORY_MAPPED_VIEW_ADDRESS,
        pub ring: Ring,
    }

    unsafe impl Send for Mapping {}
    unsafe impl Sync for Mapping {}

    impl Mapping {
        /// Creates the shared memory mapping with permissions for System, Services, and All Users.
        pub fn create() -> Option<Self> {
            unsafe {
                let mut sd = PSECURITY_DESCRIPTOR::default();
                // SDDL: SYSTEM (SY), LocalService (LS), AuthenticatedUsers (AU), World/Everyone (WD) Full Control
                ConvertStringSecurityDescriptorToSecurityDescriptorW(
                    w!("D:(A;;GA;;;SY)(A;;GA;;;LS)(A;;GA;;;AU)(A;;GA;;;WD)S:(ML;;NW;;;LW)"),
                    SDDL_REVISION_1,
                    &mut sd,
                    None,
                )
                .ok()?;
                let sa = SECURITY_ATTRIBUTES {
                    nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                    lpSecurityDescriptor: sd.0,
                    bInheritHandle: false.into(),
                };
                let handle = CreateFileMappingW(
                    INVALID_HANDLE_VALUE,
                    Some(&sa),
                    PAGE_READWRITE,
                    0,
                    TOTAL as u32,
                    &HSTRING::from(NAME),
                );
                let existed = GetLastError() == ERROR_ALREADY_EXISTS;
                let _ = LocalFree(HLOCAL(sd.0));
                let mapping = Self::map(handle.ok()?)?;
                if !existed || !mapping.ring.is_ready() {
                    mapping.ring.init();
                }
                Some(mapping)
            }
        }

        /// Opens the existing shared memory mapping.
        pub fn open() -> Option<Self> {
            unsafe {
                Self::map(
                    OpenFileMappingW(FILE_MAP_ALL_ACCESS.0, false, &HSTRING::from(NAME)).ok()?,
                )
            }
        }

        unsafe fn map(handle: HANDLE) -> Option<Self> {
            let view = unsafe { MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, TOTAL) };
            if view.Value.is_null() {
                unsafe {
                    let _ = CloseHandle(handle);
                }
                return None;
            }
            let ring = unsafe { Ring::new(view.Value as *mut u8) };
            Some(Self { handle, view, ring })
        }
    }

    impl Drop for Mapping {
        fn drop(&mut self) {
            unsafe {
                let _ = UnmapViewOfFile(self.view);
                let _ = CloseHandle(self.handle);
            }
        }
    }
}
