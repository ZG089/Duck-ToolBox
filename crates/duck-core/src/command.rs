use serde::Serialize;
use serde_json::Value;

use crate::{paths::AppPaths, sysroot::Sysroot};

/// Everything a feature needs to execute a command.
#[derive(Debug, Clone)]
pub struct Context {
    pub paths: AppPaths,
    pub sysroot: Sysroot,
}

/// Successful command output. `command` is the dotted name reported to the WebUI.
#[derive(Debug)]
pub struct CommandOutput {
    pub command: &'static str,
    pub data: Value,
}

/// A failed command with a stable, machine-readable `code`.
#[derive(Debug)]
pub struct CommandFailure {
    pub command: &'static str,
    pub code: &'static str,
    pub message: String,
    pub details: Option<Value>,
}

pub type CommandResult = Result<CommandOutput, CommandFailure>;

/// An error plus optional structured details (for example the path of a partially
/// written artifact) that the WebUI can show next to the message.
#[derive(Debug)]
pub struct Failure {
    pub error: anyhow::Error,
    pub details: Option<Value>,
}

impl<E> From<E> for Failure
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        Self {
            error: error.into(),
            details: None,
        }
    }
}

impl Failure {
    pub fn with_details(error: impl Into<anyhow::Error>, details: Value) -> Self {
        Self {
            error: error.into(),
            details: Some(details),
        }
    }
}

impl CommandOutput {
    pub fn new(command: &'static str, data: impl Serialize) -> CommandResult {
        match serde_json::to_value(data) {
            Ok(data) => Ok(Self { command, data }),
            Err(error) => Err(CommandFailure::new(
                command,
                "serialize_error",
                &anyhow::Error::new(error),
                None,
            )),
        }
    }
}

impl CommandFailure {
    pub fn new(
        command: &'static str,
        code: &'static str,
        error: &anyhow::Error,
        details: Option<Value>,
    ) -> Self {
        Self {
            command,
            code,
            message: format!("{error:#}"),
            details,
        }
    }
}

/// Converts feature results into [`CommandResult`]s.
///
/// `classify` maps an error to its stable code; each feature owns its own error codes so
/// the runtime never needs to know about feature-specific error types.
pub trait IntoCommandResult {
    fn into_command(
        self,
        command: &'static str,
        classify: fn(&anyhow::Error) -> &'static str,
    ) -> CommandResult;
}

impl<T: Serialize> IntoCommandResult for anyhow::Result<T> {
    fn into_command(
        self,
        command: &'static str,
        classify: fn(&anyhow::Error) -> &'static str,
    ) -> CommandResult {
        match self {
            Ok(data) => CommandOutput::new(command, data),
            Err(error) => Err(CommandFailure::new(command, classify(&error), &error, None)),
        }
    }
}

impl<T: Serialize> IntoCommandResult for Result<T, Failure> {
    fn into_command(
        self,
        command: &'static str,
        classify: fn(&anyhow::Error) -> &'static str,
    ) -> CommandResult {
        match self {
            Ok(data) => CommandOutput::new(command, data),
            Err(failure) => Err(CommandFailure::new(
                command,
                classify(&failure.error),
                &failure.error,
                failure.details,
            )),
        }
    }
}

/// Default classifier for features without dedicated error codes.
pub fn internal_error(_: &anyhow::Error) -> &'static str {
    "internal_error"
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Failure, IntoCommandResult, internal_error};

    #[test]
    fn anyhow_errors_keep_their_context_chain() {
        let result: anyhow::Result<()> =
            Err(anyhow::anyhow!("root cause").context("while reading config"));
        let failure = result
            .into_command("demo.read", internal_error)
            .unwrap_err();

        assert_eq!(failure.command, "demo.read");
        assert_eq!(failure.code, "internal_error");
        assert_eq!(failure.message, "while reading config: root cause");
    }

    #[test]
    fn failures_carry_details() {
        let result: Result<(), Failure> = Err(Failure::with_details(
            anyhow::anyhow!("boom"),
            json!({ "path": "/tmp/x" }),
        ));
        let failure = result.into_command("demo.write", |_| "custom").unwrap_err();

        assert_eq!(failure.code, "custom");
        assert_eq!(failure.details, Some(json!({ "path": "/tmp/x" })));
    }
}
