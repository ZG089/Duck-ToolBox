//! Keybox installation for whichever backend is active.
//!
//! Every source (local file, bundled AOSP, generated, URL, custom provider, keybox repo)
//! funnels through [`install`], which cleans, validates, enforces the backend's own keybox
//! rules, backs up the previous file and writes the new one atomically.

use std::fs;

use anyhow::Result;
use duck_core::{
    Context, Sysroot,
    fs::{backup_copy, modified_unix, write_bytes_preserving},
};
use serde::Serialize;

use crate::{
    error::TrickyError,
    model::{Backend, KeyboxStatus},
    service::active_adapter,
};

pub mod decode;
pub mod generate;
pub mod validate;

use validate::KeyboxSummary;

/// AOSP's software attestation keys and certificates, byte-identical to
/// `system/keymaster/contexts/soft_attestation_cert.cpp` (the "AOSP keybox").
pub const AOSP_KEYBOX: &str = include_str!("../../assets/aosp-keybox.xml");

#[derive(Debug, Clone, Serialize)]
pub struct KeyboxInstallData {
    pub backend: Backend,
    pub target_path: String,
    pub backup_path: Option<String>,
    pub size: u64,
    pub source: String,
    pub summary: KeyboxSummary,
}

pub fn status(sysroot: &Sysroot, device_path: &str) -> KeyboxStatus {
    let metadata = fs::metadata(sysroot.path(device_path)).ok();
    KeyboxStatus {
        path: device_path.to_owned(),
        exists: metadata.as_ref().is_some_and(fs::Metadata::is_file),
        size: metadata.as_ref().map_or(0, fs::Metadata::len),
        modified_unix: metadata.as_ref().map_or(0, modified_unix),
    }
}

pub fn install(ctx: &Context, content: &str, source: &str) -> Result<KeyboxInstallData> {
    let (detection, adapter) = active_adapter(&ctx.sysroot)?;
    let cleaned = validate::clean(content)?;
    let summary = validate::validate(&cleaned)?;
    check_backend_rules(detection.backend, &summary)?;

    let device_path = adapter.keybox_path(&ctx.sysroot);
    let host_path = ctx.sysroot.path(&device_path);
    let existed = host_path.is_file();
    let backup = backup_copy(&host_path)?;
    write_bytes_preserving(&host_path, cleaned.as_bytes())?;
    if !existed {
        set_mode(&host_path, 0o644);
    }

    Ok(KeyboxInstallData {
        backend: detection.backend,
        target_path: device_path,
        backup_path: backup.map(|path| ctx.sysroot.display_path(&path)),
        size: cleaned.len() as u64,
        source: source.to_owned(),
        summary,
    })
}

/// TEESimulator and OhMyKeymint refuse keyboxes that lack either algorithm; TEESimulator
/// also needs full chains (leaf + issuer). Rejecting here avoids leaving the daemon on an
/// unusable file.
fn check_backend_rules(backend: Backend, summary: &KeyboxSummary) -> Result<()> {
    let needs_both = matches!(backend, Backend::TeeSimulator | Backend::OhMyKeymint);
    if needs_both && !(summary.has_ecdsa && summary.has_rsa) {
        return Err(TrickyError::BackendRule(format!(
            "{} requires a keybox with both an ECDSA and an RSA key",
            backend.identity()
        ))
        .into());
    }
    if backend == Backend::TeeSimulator && summary.chain_lengths.iter().any(|len| *len < 2) {
        return Err(TrickyError::BackendRule(
            "TEESimulator requires certificate chains with at least two certificates".into(),
        )
        .into());
    }
    Ok(())
}

/// Keyboxes are a few KiB; anything larger is not a keybox.
const MAX_KEYBOX_BYTES: usize = 1024 * 1024;

pub async fn fetch(url: &str, decode_steps: &str) -> Result<String> {
    decode::check(decode_steps)?;
    if !duck_platform::net::is_web_url(url) {
        return Err(TrickyError::Download(format!("not an http(s) URL: `{url}`")).into());
    }
    let body = duck_platform::net::fetch_bytes(url, MAX_KEYBOX_BYTES)
        .await
        .map_err(|error| TrickyError::Download(format!("{error:#}")))?;

    let decoded = decode::apply(decode_steps, &body)?;
    String::from_utf8(decoded).map_err(|_| {
        TrickyError::InvalidKeybox("downloaded keybox is not UTF-8 text".into()).into()
    })
}

#[cfg(unix)]
fn set_mode(path: &std::path::Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode));
}

#[cfg(not(unix))]
fn set_mode(_path: &std::path::Path, _mode: u32) {}

#[cfg(test)]
mod tests {
    use super::{AOSP_KEYBOX, check_backend_rules, validate};
    use crate::model::Backend;

    #[test]
    fn aosp_keybox_is_accepted_by_every_backend() {
        let summary = validate::validate(AOSP_KEYBOX).unwrap();
        for backend in [
            Backend::TrickyStore,
            Backend::TrickyStoreLegacy,
            Backend::TeeSimulator,
            Backend::OhMyKeymint,
        ] {
            check_backend_rules(backend, &summary).unwrap();
        }
    }

    #[test]
    fn ecdsa_only_keybox_is_rejected_for_tee_simulator() {
        let summary = validate::KeyboxSummary {
            keyboxes: 1,
            has_ecdsa: true,
            has_rsa: false,
            chain_lengths: vec![3],
        };
        check_backend_rules(Backend::TrickyStore, &summary).unwrap();
        assert!(check_backend_rules(Backend::TeeSimulator, &summary).is_err());
        assert!(check_backend_rules(Backend::OhMyKeymint, &summary).is_err());
    }
}
