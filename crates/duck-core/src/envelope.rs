//! The JSON envelope every command prints on stdout.
//!
//! The WebUI parses exactly one envelope per invocation. `api` is bumped only when the
//! envelope shape itself changes so an outdated WebUI can explain the mismatch instead of
//! misreading data.

use serde_json::{Value, json};

use crate::{
    command::{CommandFailure, CommandOutput},
    unix_now,
};

pub const API_VERSION: u32 = 1;

pub fn success(output: &CommandOutput) -> Value {
    json!({
        "ok": true,
        "api": API_VERSION,
        "command": output.command,
        "data": output.data,
        "error": Value::Null,
        "ts": unix_now(),
    })
}

pub fn failure(failure: &CommandFailure) -> Value {
    let mut error = json!({
        "code": failure.code,
        "message": failure.message,
    });
    if let Some(details) = &failure.details {
        error["details"] = details.clone();
    }

    json!({
        "ok": false,
        "api": API_VERSION,
        "command": failure.command,
        "data": Value::Null,
        "error": error,
        "ts": unix_now(),
    })
}

pub fn emit(value: &Value) {
    println!(
        "{}",
        serde_json::to_string(value).unwrap_or_else(|_| value.to_string())
    );
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{API_VERSION, failure, success};
    use crate::command::{CommandFailure, CommandOutput};

    #[test]
    fn success_envelope_has_stable_shape() {
        let value = success(&CommandOutput {
            command: "demo.ok",
            data: json!({ "answer": 42 }),
        });

        assert_eq!(value["ok"], true);
        assert_eq!(value["api"], API_VERSION);
        assert_eq!(value["command"], "demo.ok");
        assert_eq!(value["data"]["answer"], 42);
        assert!(value["error"].is_null());
    }

    #[test]
    fn failure_envelope_omits_missing_details() {
        let value = failure(&CommandFailure {
            command: "demo.fail",
            code: "bad_input",
            message: "nope".into(),
            details: None,
        });

        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], "bad_input");
        assert!(value["error"].get("details").is_none());
    }
}
