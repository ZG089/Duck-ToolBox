use std::{fs::OpenOptions, io::Write};

use serde_json::{Value, json};

use crate::{paths::AppPaths, unix_now};

const MAX_LOG_BYTES: u64 = 512 * 1024;

/// Appends one JSON line to `duckd.log`, rotating it once it grows past 512 KiB.
/// Logging is best effort: failures never affect the command result.
pub fn append(paths: &AppPaths, value: &Value) {
    if paths.ensure_runtime_dirs().is_err() {
        return;
    }

    let mut line = value.clone();
    if let Some(object) = line.as_object_mut()
        && !object.contains_key("ts")
    {
        object.insert("ts".into(), json!(unix_now()));
    }

    rotate_if_needed(paths);

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&paths.log_path)
    {
        let _ = writeln!(file, "{line}");
    }
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
