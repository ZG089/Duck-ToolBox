//! Thin FFI wrapper over Qualcomm's `libQSEEComAPI.so`.
//!
//! This is the only module in Duck ToolBox permitted to use `unsafe`: it dereferences the
//! shared ION buffer owned by the trusted application. Everything else is safe Rust.

use std::{
    ffi::CString,
    os::raw::{c_char, c_int, c_void},
    ptr, slice, thread,
    time::Duration,
};

use anyhow::{Context, Result, anyhow, bail};
use duck_platform::props;
use libloading::Library;

const SHARED_BUF_SIZE: usize = 0xA000;
/// `QSEECOM_ALIGN_SIZE` from the kernel's `qseecom_kernel.h`. AOSP's Qualcomm keymaster HAL
/// (`hardware/qcom/keymaster`) places the response at `QSEECOM_ALIGN(command length)` so
/// command and response never share a cache line of the shared buffer.
const QSEECOM_ALIGN_SIZE: usize = 0x40;
const DEFAULT_LIB_PATH: &str = "/vendor/lib64/libQSEEComAPI.so";
const DEFAULT_LIB_PATH_ALT: &str = "/vendor/lib64/hw/libQSEEComAPI.so";
const FALLBACK_TA_NAME: &str = "keymaster";

const CMD_GET_VERSION: u32 = 0x0200;
const CMD_SET_VERSION: u32 = 0x0207;
const CMD_SET_PROVISIONING_DEVICE_ID_SUCCESS: u32 = 0x2218;

#[repr(C)]
struct QseeComHandle {
    ion_sbuffer: *mut u8,
}

type StartApp =
    unsafe extern "C" fn(*mut *mut QseeComHandle, *const c_char, *const c_char, u32) -> c_int;
type SendCmd =
    unsafe extern "C" fn(*mut QseeComHandle, *mut c_void, u32, *mut c_void, u32) -> c_int;
type ShutdownApp = unsafe extern "C" fn(*mut *mut QseeComHandle) -> c_int;

/// Metadata gathered while talking to the TA, surfaced in the provisioning report.
#[derive(Debug, Default, Clone)]
pub(crate) struct SessionInfo {
    pub(crate) loaded_library: Option<String>,
    pub(crate) ta_api_version: Option<String>,
    pub(crate) ta_version: Option<String>,
    pub(crate) response_hex: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct KmVersion {
    ta_api_major: u32,
    ta_api_minor: u32,
    ta_major: u32,
    ta_minor: u32,
}

/// Loads the API, starts the TA, sends the provisioning command and marks it successful.
pub(crate) fn provision(ta_path: &str, ta_name: &str, command: &[u8]) -> Result<SessionInfo> {
    let api = QseecomApi::load()?;
    wait_listeners();

    let session = api.start_session(ta_path, ta_name)?;
    let version = session.get_version()?;
    session.set_version()?;

    let response = session.send(command)?;
    if response.len() < 8 {
        bail!("PROVISION_DEVICE_IDS returned too little data");
    }
    let status = read_i32(&response, 0)?;
    if status != 0 {
        bail!("PROVISION_DEVICE_IDS failed with status {status}");
    }
    let data_len = read_u32(&response, 4)? as usize;
    let available = response.len().saturating_sub(8);
    let response_hex = (data_len > 0)
        .then(|| hex::encode(&response[8..8 + data_len.min(available)]))
        .filter(|hex| !hex.is_empty());

    session.set_success_marker()?;

    Ok(SessionInfo {
        loaded_library: Some(api.loaded_path.clone()),
        ta_api_version: Some(format!("{}.{}", version.ta_api_major, version.ta_api_minor)),
        ta_version: Some(format!("{}.{}", version.ta_major, version.ta_minor)),
        response_hex,
    })
}

fn wait_listeners() {
    for _ in 0..50 {
        if props::get("vendor.sys.listeners.registered") == "true" {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
}

struct QseecomApi {
    _library: Library,
    start_app: StartApp,
    send_cmd: SendCmd,
    shutdown_app: ShutdownApp,
    loaded_path: String,
}

impl QseecomApi {
    fn load() -> Result<Self> {
        for candidate in [DEFAULT_LIB_PATH, DEFAULT_LIB_PATH_ALT] {
            let Ok(library) = (unsafe { Library::new(candidate) }) else {
                continue;
            };

            let start_app = *unsafe { library.get::<StartApp>(b"QSEECom_start_app\0") }
                .context("resolve QSEECom_start_app")?;
            let send_cmd = *unsafe { library.get::<SendCmd>(b"QSEECom_send_cmd\0") }
                .context("resolve QSEECom_send_cmd")?;
            let shutdown_app = *unsafe { library.get::<ShutdownApp>(b"QSEECom_shutdown_app\0") }
                .context("resolve QSEECom_shutdown_app")?;

            return Ok(Self {
                _library: library,
                start_app,
                send_cmd,
                shutdown_app,
                loaded_path: candidate.into(),
            });
        }

        bail!("failed to load QSEEComAPI from {DEFAULT_LIB_PATH} or {DEFAULT_LIB_PATH_ALT}")
    }

    fn start_session(&self, ta_path: &str, ta_name: &str) -> Result<QseecomSession<'_>> {
        let handle = self.try_start(ta_path, ta_name).or_else(|error| {
            if ta_name.trim() != FALLBACK_TA_NAME {
                self.try_start(ta_path, FALLBACK_TA_NAME)
                    .with_context(|| format!("fallback to {FALLBACK_TA_NAME} after {error}"))
            } else {
                Err(error)
            }
        })?;

        Ok(QseecomSession { api: self, handle })
    }

    fn try_start(&self, ta_path: &str, ta_name: &str) -> Result<*mut QseeComHandle> {
        let path = CString::new(ta_path.trim()).context("TA path contains NUL byte")?;
        let name = CString::new(ta_name.trim()).context("TA name contains NUL byte")?;
        let mut handle = ptr::null_mut();
        let status = unsafe {
            (self.start_app)(
                &mut handle,
                path.as_ptr(),
                name.as_ptr(),
                SHARED_BUF_SIZE as u32,
            )
        };
        if status != 0 || handle.is_null() {
            bail!("QSEECom_start_app failed with status {status}");
        }

        Ok(handle)
    }
}

struct QseecomSession<'a> {
    api: &'a QseecomApi,
    handle: *mut QseeComHandle,
}

impl QseecomSession<'_> {
    fn get_version(&self) -> Result<KmVersion> {
        let response = self.send(&CMD_GET_VERSION.to_le_bytes())?;
        if response.len() < 20 {
            bail!("GET_VERSION returned too little data");
        }
        if read_i32(&response, 0)? != 0 {
            bail!("GET_VERSION failed with status {}", read_i32(&response, 0)?);
        }

        Ok(KmVersion {
            ta_api_major: read_u32(&response, 4)?,
            ta_api_minor: read_u32(&response, 8)?,
            ta_major: read_u32(&response, 12)?,
            ta_minor: read_u32(&response, 16)?,
        })
    }

    fn set_version(&self) -> Result<()> {
        let mut request = Vec::with_capacity(24);
        for value in [CMD_SET_VERSION, 4, 5, 4, 5, 0_u32] {
            request.extend_from_slice(&value.to_le_bytes());
        }
        let response = self.send(&request)?;
        if read_i32(&response, 0)? != 0 {
            bail!("SET_VERSION failed with status {}", read_i32(&response, 0)?);
        }
        Ok(())
    }

    fn set_success_marker(&self) -> Result<()> {
        let response = self.send(&CMD_SET_PROVISIONING_DEVICE_ID_SUCCESS.to_le_bytes())?;
        if read_i32(&response, 0)? != 0 {
            bail!(
                "SET_PROVISIONING_DEVICE_ID_SUCCESS failed with status {}",
                read_i32(&response, 0)?
            );
        }
        Ok(())
    }

    fn send(&self, request: &[u8]) -> Result<Vec<u8>> {
        let rsp_offset = qseecom_align(request.len());
        if rsp_offset >= SHARED_BUF_SIZE {
            bail!("request is too large for QSEECom shared buffer");
        }

        let buffer = unsafe {
            let sbuffer = (*self.handle).ion_sbuffer;
            if sbuffer.is_null() {
                bail!("QSEECom shared buffer is unavailable");
            }
            slice::from_raw_parts_mut(sbuffer, SHARED_BUF_SIZE)
        };
        buffer.fill(0);
        buffer[..request.len()].copy_from_slice(request);

        let response_len = (SHARED_BUF_SIZE - rsp_offset) as u32;
        let status = unsafe {
            (self.api.send_cmd)(
                self.handle,
                buffer.as_mut_ptr().cast(),
                request.len() as u32,
                buffer[rsp_offset..].as_mut_ptr().cast(),
                response_len,
            )
        };
        if status != 0 {
            bail!("QSEECom_send_cmd failed with status {status}");
        }

        Ok(buffer[rsp_offset..].to_vec())
    }
}

impl Drop for QseecomSession<'_> {
    fn drop(&mut self) {
        let mut handle = self.handle;
        unsafe {
            let _ = (self.api.shutdown_app)(&mut handle);
        }
    }
}

/// The kernel's `QSEECOM_ALIGN(x)`: round up to the next multiple of 64.
fn qseecom_align(value: usize) -> usize {
    (value + QSEECOM_ALIGN_SIZE - 1) & !(QSEECOM_ALIGN_SIZE - 1)
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let chunk = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| anyhow!("missing u32 at offset {offset}"))?;
    Ok(u32::from_le_bytes(
        chunk.try_into().expect("slice is 4 bytes"),
    ))
}

fn read_i32(bytes: &[u8], offset: usize) -> Result<i32> {
    let chunk = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| anyhow!("missing i32 at offset {offset}"))?;
    Ok(i32::from_le_bytes(
        chunk.try_into().expect("slice is 4 bytes"),
    ))
}

#[cfg(test)]
mod tests {
    use super::{SHARED_BUF_SIZE, qseecom_align};

    #[test]
    fn aligns_like_the_kernel_macro() {
        assert_eq!(qseecom_align(0), 0);
        assert_eq!(qseecom_align(4), 64);
        assert_eq!(qseecom_align(64), 64);
        assert_eq!(qseecom_align(65), 128);
    }

    #[test]
    fn request_and_response_fit_the_driver_check() {
        // qseecom rejects cmd_req_len + resp_len > sb_length.
        for len in [4, 24, 63, 64, 1000] {
            let offset = qseecom_align(len);
            assert!(len + (SHARED_BUF_SIZE - offset) <= SHARED_BUF_SIZE);
            assert_eq!(offset % 64, 0);
        }
    }
}
