//! Prop settings: the sensitive-prop handler and the Verified Boot hash.
//!
//! File locations match Tricky Addon (`/data/adb/boot_hash`,
//! `/data/adb/disable_prop_handler`) so both tools share one configuration. Props are
//! changed with `resetprop -n`, which the KernelSU module guide requires in boot scripts.

use anyhow::Result;
use duck_core::{Context, Sysroot, fs::write_string_atomic};
use duck_platform::props;
use serde::{Deserialize, Serialize};

use crate::{error::TrickyError, model::PropStatus};

pub const BOOT_HASH_FILE: &str = "/data/adb/boot_hash";
pub const DISABLE_PROP_HANDLER_FILE: &str = "/data/adb/disable_prop_handler";
const VBMETA_DIGEST: &str = "ro.boot.vbmeta.digest";

#[derive(Debug, Deserialize)]
pub struct PropRequest {
    pub prop_handler_enabled: bool,
    #[serde(default)]
    pub boot_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PropData {
    pub prop_handler_enabled: bool,
    pub boot_hash: Option<String>,
    pub applied_boot_hash: bool,
}

pub fn status(sysroot: &Sysroot) -> PropStatus {
    PropStatus {
        prop_handler_enabled: !sysroot.path(DISABLE_PROP_HANDLER_FILE).exists(),
        boot_hash: read_boot_hash(sysroot),
    }
}

pub fn save(ctx: &Context, request: PropRequest) -> Result<PropData> {
    let sysroot = &ctx.sysroot;
    let disable_path = sysroot.path(DISABLE_PROP_HANDLER_FILE);
    if request.prop_handler_enabled {
        let _ = std::fs::remove_file(&disable_path);
    } else {
        write_string_atomic(&disable_path, "")?;
    }

    let mut applied_boot_hash = false;
    let boot_hash = match request.boot_hash.as_deref().map(str::trim) {
        Some(hash) if !hash.is_empty() => {
            let hash = normalize_boot_hash(hash)?;
            write_string_atomic(&sysroot.path(BOOT_HASH_FILE), &hash)?;
            // Only touch live properties when managing the running system, not a test sysroot.
            if sysroot.root() == std::path::Path::new("/") {
                props::reset(VBMETA_DIGEST, &hash)?;
                applied_boot_hash = true;
            }
            Some(hash)
        }
        _ => {
            let _ = std::fs::remove_file(sysroot.path(BOOT_HASH_FILE));
            None
        }
    };

    Ok(PropData {
        prop_handler_enabled: request.prop_handler_enabled,
        boot_hash,
        applied_boot_hash,
    })
}

fn read_boot_hash(sysroot: &Sysroot) -> Option<String> {
    let raw = duck_core::fs::read_optional(&sysroot.path(BOOT_HASH_FILE)).ok()??;
    let hash: String = raw
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .flat_map(str::chars)
        .filter(|ch| !ch.is_whitespace())
        .collect();
    (!hash.is_empty()).then(|| hash.to_ascii_lowercase())
}

/// The `verifiedBootHash` from KeyAttestation is a 64-character lowercase hex string.
fn normalize_boot_hash(hash: &str) -> Result<String> {
    let hash: String = hash.chars().filter(|ch| !ch.is_whitespace()).collect();
    let hash = hash.to_ascii_lowercase();
    if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(TrickyError::InvalidPolicy {
            field: "boot_hash".into(),
            reason: "expected 64 hexadecimal characters".into(),
        }
        .into());
    }
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use duck_core::Sysroot;

    use super::{DISABLE_PROP_HANDLER_FILE, normalize_boot_hash, read_boot_hash, status};

    #[test]
    fn normalizes_and_validates_boot_hash() {
        assert_eq!(
            normalize_boot_hash(&format!(" {} ", "AB".repeat(32))).unwrap(),
            "ab".repeat(32)
        );
        assert!(normalize_boot_hash("abc").is_err());
        assert!(normalize_boot_hash(&"z".repeat(64)).is_err());
    }

    #[test]
    fn status_reflects_disable_file() {
        let root = std::env::temp_dir().join(format!("duck-props-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let sysroot = Sysroot::new(&root);
        assert!(status(&sysroot).prop_handler_enabled);

        let disable = sysroot.path(DISABLE_PROP_HANDLER_FILE);
        fs::create_dir_all(disable.parent().unwrap()).unwrap();
        fs::write(&disable, "").unwrap();
        assert!(!status(&sysroot).prop_handler_enabled);
    }

    #[test]
    fn reads_boot_hash_ignoring_comments() {
        let root = std::env::temp_dir().join(format!("duck-props-read-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let sysroot = Sysroot::new(&root);
        let path = sysroot.path(super::BOOT_HASH_FILE);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, format!("# note\n{}\n", "CD".repeat(32))).unwrap();

        assert_eq!(read_boot_hash(&sysroot), Some("cd".repeat(32)));
    }
}
