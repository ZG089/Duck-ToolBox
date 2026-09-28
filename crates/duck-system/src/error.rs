use thiserror::Error;

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("no supported root manager (KernelSU, APatch or Magisk) was found")]
    NoRootManager,
    #[error("only http(s) links can be opened: {0}")]
    InvalidUrl(String),
    #[error("no update is available on the {0} channel")]
    NoUpdate(&'static str),
    #[error("update package rejected: {0}")]
    InvalidPackage(String),
    #[error("download failed: {0}")]
    Download(String),
}

/// Stable error codes for the WebUI.
pub fn code_of(error: &anyhow::Error) -> &'static str {
    match error.downcast_ref::<SystemError>() {
        Some(SystemError::NoRootManager) => "no_root_manager",
        Some(SystemError::InvalidUrl(_)) => "invalid_url",
        Some(SystemError::NoUpdate(_)) => "no_update",
        Some(SystemError::InvalidPackage(_)) => "invalid_package",
        Some(SystemError::Download(_)) => "download_failed",
        None => "internal_error",
    }
}
