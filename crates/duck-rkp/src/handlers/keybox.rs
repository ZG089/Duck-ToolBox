use std::fs;

use anyhow::{Context, anyhow};
use duck_core::{AppPaths, Failure};
use duck_platform::net::http_client;

use crate::{
    cli::KeyboxArgs,
    cose::{DeviceKeys, build_csr, generate_ec_keypair},
    crypto_kdf::resolve_seed,
    http::{fetch_eek, submit_csr},
    keybox_xml::{build_keybox_xml, parse_der_cert_chain, summarize_chain},
    output::{KeyboxData, resolve_keybox_output_path},
};

use super::{ensure_request_context, path_details, random_bytes, resolve_runtime, write_output};

pub(crate) async fn run(paths: &AppPaths, args: &KeyboxArgs) -> Result<KeyboxData, Failure> {
    let resolved = resolve_runtime(paths, &args.shared, Some(1), args.output.clone())?;
    let server_url = ensure_request_context(&resolved.profile)?;
    let keys = DeviceKeys::from_seed_with_curve(
        resolve_seed(&resolved.profile.key_source)?,
        resolved.profile.curve,
    );
    let client = http_client()?;
    let ec_key = generate_ec_keypair()?;

    let eek = fetch_eek(&client, &resolved.profile.fingerprint.value, &server_url).await?;
    let csr = build_csr(
        &keys,
        &eek.challenge,
        std::slice::from_ref(&ec_key.cose_public),
        &eek.eek_public,
        &eek.eek_id,
        &resolved.profile.device,
        eek.eek_curve,
    )?;

    let keybox_path =
        resolve_keybox_output_path(paths, &resolved, args.output.is_some()).map_err(|error| {
            Failure::with_details(error, path_details("output_path", &resolved.output_path))
        })?;
    if let Some(parent) = keybox_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let csr_path = keybox_path.with_extension("cbor");
    let csr_details = path_details("csr_path", &csr_path);
    write_output(&csr_path, &csr.csr_bytes, Some(csr_details.clone()))?;

    let cert_chains = submit_csr(&client, &csr.csr_bytes, &eek.challenge, &server_url)
        .await
        .map_err(|error| Failure::with_details(error, csr_details.clone()))?;
    let first_chain = cert_chains.first().ok_or_else(|| {
        Failure::with_details(
            anyhow!("RKP server returned no certificate chains"),
            csr_details,
        )
    })?;

    let parsed = parse_der_cert_chain(first_chain)?;
    let device_id = random_device_id(&resolved.profile.device.manufacturer)?;
    let xml = build_keybox_xml(&ec_key.secret_key, &parsed, &device_id)?;
    write_output(&keybox_path, xml.as_bytes(), None)?;

    Ok(KeyboxData {
        mode: resolved.profile.key_source.mode_label().into(),
        cdi_leaf_pubkey_hex: keys.public_key_hex(),
        challenge_hex: eek.challenge_hex,
        csr_path: csr_path.display().to_string(),
        keybox_path: keybox_path.display().to_string(),
        keybox_xml: xml,
        device_id,
        chain_summary: summarize_chain(&parsed),
    })
}

fn random_device_id(manufacturer: &str) -> anyhow::Result<String> {
    let manufacturer = manufacturer.trim();
    let manufacturer = if manufacturer.is_empty() {
        "generic"
    } else {
        manufacturer
    };
    Ok(format!("{manufacturer}-{}", hex::encode(random_bytes(6)?)))
}
