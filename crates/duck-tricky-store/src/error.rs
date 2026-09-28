use thiserror::Error;

#[derive(Debug, Error)]
pub enum TrickyError {
    #[error(
        "no supported keystore module (Tricky Store, TEESimulator or OhMyKeymint) is installed"
    )]
    NoBackend,
    #[error("invalid keybox: {0}")]
    InvalidKeybox(String),
    #[error("unsupported decode step `{0}`; allowed steps are cat, base64 -d and xxd -r -p")]
    UnsupportedDecoder(String),
    #[error("invalid policy value for `{field}`: {reason}")]
    InvalidPolicy { field: String, reason: String },
    #[error("{0}")]
    BackendRule(String),
    #[error("download failed: {0}")]
    Download(String),
}

impl TrickyError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoBackend => "no_backend",
            Self::InvalidKeybox(_) => "invalid_keybox",
            Self::UnsupportedDecoder(_) => "unsupported_decoder",
            Self::InvalidPolicy { .. } => "invalid_policy",
            Self::BackendRule(_) => "backend_rule",
            Self::Download(_) => "download_failed",
        }
    }
}

pub(crate) fn code_of(error: &anyhow::Error) -> &'static str {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<TrickyError>())
        .map_or("internal_error", TrickyError::code)
}
