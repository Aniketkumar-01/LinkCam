//! LinkCam Virtual Camera on Windows 11: a Media Foundation Media Source COM DLL
//! loaded by Windows Camera Frame Server, serving pre-decoded NV12 frames from shared memory.

pub mod shared;

#[cfg(windows)]
mod activate;
#[cfg(windows)]
pub mod placeholder;
#[cfg(windows)]
pub mod register;
#[cfg(windows)]
mod source;
#[cfg(windows)]
mod stream;

/// Media Source COM Class ID: {C823B5A1-3E2A-4F75-9C47-D572B28B4592}
pub const CLSID: u128 = 0xc823b5a1_3e2a_4f75_9c47_d572b28b4592;

#[cfg(windows)]
mod exports {
    use crate::activate::Factory;
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicIsize, Ordering};
    use windows::Win32::Foundation::{
        CLASS_E_CLASSNOTAVAILABLE, E_FAIL, E_POINTER, HMODULE, S_FALSE, S_OK,
    };
    use windows::Win32::System::Com::IClassFactory;
    use windows::Win32::System::LibraryLoader::{
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        GetModuleFileNameW, GetModuleHandleExW,
    };
    use windows::core::{GUID, HRESULT, Interface, PCWSTR};

    pub(crate) static LIVE: AtomicIsize = AtomicIsize::new(0);

    #[no_mangle]
    extern "system" fn DllGetClassObject(
        clsid: *const GUID,
        iid: *const GUID,
        out: *mut *mut c_void,
    ) -> HRESULT {
        if out.is_null() || clsid.is_null() || iid.is_null() {
            return E_POINTER;
        }
        unsafe { *out = std::ptr::null_mut() };
        if unsafe { *clsid } != GUID::from_u128(crate::CLSID) {
            return CLASS_E_CLASSNOTAVAILABLE;
        }
        let factory: IClassFactory = Factory.into();
        unsafe { factory.query(iid, out) }
    }

    #[no_mangle]
    extern "system" fn DllCanUnloadNow() -> HRESULT {
        if LIVE.load(Ordering::Acquire) == 0 {
            S_OK
        } else {
            S_FALSE
        }
    }

    fn own_path() -> Option<std::path::PathBuf> {
        let mut module = HMODULE::default();
        let mut buf = [0u16; 1024];
        unsafe {
            GetModuleHandleExW(
                GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS
                    | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                PCWSTR(own_path as *const u16),
                &mut module,
            )
            .ok()?;
            let n = GetModuleFileNameW(Some(module), &mut buf) as usize;
            (n > 0).then(|| String::from_utf16_lossy(&buf[..n]).into())
        }
    }

    #[no_mangle]
    extern "system" fn DllRegisterServer() -> HRESULT {
        match own_path().map(|p| crate::register::register_class(&p)) {
            Some(Ok(())) => S_OK,
            Some(Err(e)) => e.code(),
            None => E_FAIL,
        }
    }

    #[no_mangle]
    extern "system" fn DllUnregisterServer() -> HRESULT {
        crate::register::unregister_class().map_or_else(|e| e.code(), |()| S_OK)
    }
}
