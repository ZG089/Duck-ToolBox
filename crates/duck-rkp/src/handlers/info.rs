use anyhow::Result;
use duck_core::AppPaths;
use serde::Serialize;

use crate::{cli::SharedRunArgs, cose::DeviceKeys, crypto_kdf::resolve_seed, profile::DeviceInfo};

use super::resolve_runtime;

#[derive(Debug, Serialize)]
pub(crate) struct InfoData {
    mode: String,
    curve: String,
    seed_hex: String,
    public_key_hex: String,
    ed25519_pubkey_hex: String,
    device: DeviceInfo,
    fingerprint: String,
    server_url: String,
    num_keys: u32,
    output_path: String,
}

pub(crate) fn run(paths: &AppPaths, args: &SharedRunArgs) -> Result<InfoData> {
    let resolved = resolve_runtime(paths, args, None, None)?;
    let profile = &resolved.profile;
    let keys = DeviceKeys::from_seed_with_curve(resolve_seed(&profile.key_source)?, profile.curve);
    let public_key_hex = keys.public_key_hex();

    Ok(InfoData {
        mode: profile.key_source.mode_label().into(),
        curve: profile.curve.as_str().into(),
        seed_hex: keys.seed_hex(),
        ed25519_pubkey_hex: public_key_hex.clone(),
        public_key_hex,
        device: profile.device.clone(),
        fingerprint: profile.fingerprint.value.clone(),
        server_url: profile.server_url.clone(),
        num_keys: profile.num_keys,
        output_path: resolved.output_path.display().to_string(),
    })
}
