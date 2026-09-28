//! Command history in `duckd.log`, one JSON line per invocation.
//!
//! Only the outcome is recorded (command, status, error), never the `data` payload, so
//! profile secrets and device identifiers do not end up in a world-readable log.

use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{paths::AppPaths, unix_now};

const MAX_LOG_BYTES: u64 = 512 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntry {
    pub ts: u64,
    pub command: String,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl LogEntry {
    pub fn from_envelope(envelope: &Value) -> Self {
        let text = |value: &Value| value.as_str().map(str::to_owned);
        Self {
            ts: envelope["ts"].as_u64().unwrap_or_else(unix_now),
            command: text(&envelope["command"]).unwrap_or_default(),
            ok: envelope["ok"].as_bool().unwrap_or(false),
            code: text(&envelope["error"]["code"]),
            message: text(&envelope["error"]["message"]),
        }
    }
}

/// Appends one entry, rotating the log once it grows past 512 KiB. Logging is best effort:
/// failures never affect the command result.
pub fn append(paths: &AppPaths, envelope: &Value) {
    if paths.ensure_runtime_dirs().is_err() {
        return;
    }
    rotate_if_needed(paths);

    let Ok(line) = serde_json::to_string(&LogEntry::from_envelope(envelope)) else {
        return;
    };
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&paths.log_path)
    {
        let _ = writeln!(file, "{line}");
    }
}

/// The newest `limit` entries, newest first. Older releases logged whole envelopes; those
/// lines are reduced to entries, and anything unparsable is skipped.
pub fn tail(paths: &AppPaths, limit: usize) -> Result<Vec<LogEntry>> {
    let file = match std::fs::File::open(&paths.log_path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    let mut entries: Vec<LogEntry> = BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| serde_json::from_str::<Value>(&line).ok())
        .filter_map(|value| {
            if value.get("error").is_some() || value.get("data").is_some() {
                Some(LogEntry::from_envelope(&value))
            } else {
                serde_json::from_value(value).ok()
            }
        })
        .collect();
    entries.reverse();
    entries.truncate(limit);
    Ok(entries)
}

fn rotate_if_needed(paths: &AppPaths) {
    let Ok(metadata) = std::fs::metadata(&paths.log_path) else {
        return;
    };
    if metadata.len() < MAX_LOG_BYTES {
        return;
    }

    let rotated = paths.logs_dir.join("duckd.log.1");
    let _ = std::fs::remove_file(&rotated);
    let _ = std::fs::rename(&paths.log_path, rotated);
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{append, tail};
    use crate::AppPaths;

    #[test]
    fn records_outcomes_without_payloads() {
        let dir = std::env::temp_dir().join(format!("duck-log-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let paths = AppPaths::for_root(&dir, &dir);

        append(
            &paths,
            &json!({ "ok": true, "command": "rkp.profile.show", "data": { "seed_hex": "secret" }, "ts": 1 }),
        );
        append(
            &paths,
            &json!({ "ok": false, "command": "rkp.keybox", "error": { "code": "bad", "message": "nope" }, "ts": 2 }),
        );

        let raw = std::fs::read_to_string(&paths.log_path).unwrap();
        assert!(!raw.contains("secret"));

        let entries = tail(&paths, 10).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].command, "rkp.keybox");
        assert_eq!(entries[0].code.as_deref(), Some("bad"));
        assert!(entries[1].ok);
        assert_eq!(tail(&paths, 1).unwrap().len(), 1);
    }
}
