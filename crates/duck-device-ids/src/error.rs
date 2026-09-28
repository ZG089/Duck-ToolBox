use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeviceIdsError {
    #[error("device field `{0}` is required before provisioning")]
    MissingField(&'static str),
}

impl DeviceIdsError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingField(_) => "missing_device_field",
        }
    }
}

pub(crate) fn code_of(error: &anyhow::Error) -> &'static str {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<DeviceIdsError>())
        .map_or("internal_error", DeviceIdsError::code)
}
