use std::collections::HashSet;

use anyhow::{Result, anyhow};
use ciborium::value::Value;

use crate::{
    cbor::{as_array, as_bytes, as_map, as_text, decode, map_get},
    cose::{CWT_ISSUER, CWT_SUBJECT, DICE_KEY_USAGE, DICE_SUBJECT_PUB_KEY, build_sig_structure},
};

use super::{
    ANDROID_DICE_PROFILE_VERSION, DICE_PROFILE_NAME,
    checks::{ensure_unique_integer_keys, required_int_bytes, required_int_text},
    cose_key::{
        CosePublicKeyInfo, parse_cose_public_key, parse_cose_sign1, verify_signed_data_signature,
    },
};

/// Open DICE payload fields that must be non-empty byte strings when present.
const OPTIONAL_DICE_BYTE_FIELDS: [(i128, &str); 6] = [
    (-4_670_545, "code hash"),
    (-4_670_546, "code descriptor"),
    (-4_670_547, "configuration hash"),
    (-4_670_549, "authority hash"),
    (-4_670_550, "authority descriptor"),
    (-4_670_551, "mode"),
];
const DICE_CONFIGURATION_DESCRIPTOR: i128 = -4_670_548;

pub(super) fn extract_uds_key(dice_chain: &[Value]) -> Result<CosePublicKeyInfo> {
    let uds_key = dice_chain
        .first()
        .ok_or_else(|| anyhow!("DICE chain is empty"))?;
    parse_cose_public_key(as_map(uds_key, "UDS COSE key")?, "UDS COSE key")
}

pub(super) fn verify_dice_chain_and_extract_leaf_key(
    dice_chain: &[Value],
) -> Result<CosePublicKeyInfo> {
    let mut signer = extract_uds_key(dice_chain)?;

    for (index, item) in dice_chain.iter().enumerate().skip(1) {
        let label = format!("DICE chain entry {index}");
        let entry = parse_cose_sign1(as_array(item, "DICE chain entry")?, &label, Some(&signer))?;
        let signature_input = build_sig_structure("Signature1", entry.protected, entry.payload)?;
        if !verify_signed_data_signature(&signer, entry.signature, &signature_input)? {
            return Err(anyhow!("{label} signature is invalid"));
        }

        signer = parse_dice_entry_payload(&decode(entry.payload)?, &label)?;
    }

    Ok(signer)
}

pub(super) fn validate_uds_certs(value: &Value) -> Result<()> {
    let entries = as_map(value, "UDS certs")?;
    let mut seen = HashSet::new();

    for (signer_name, chain) in entries {
        let signer_name = as_text(signer_name, "UDS cert signer")?;
        if signer_name.is_empty() {
            return Err(anyhow!("UDS cert signer name must not be empty"));
        }
        if !seen.insert(signer_name.to_owned()) {
            return Err(anyhow!("duplicate UDS cert signer `{signer_name}`"));
        }

        let certs = as_array(chain, &format!("UDS cert chain `{signer_name}`"))?;
        if certs.is_empty() {
            return Err(anyhow!(
                "UDS cert chain `{signer_name}` must contain at least one certificate"
            ));
        }

        for (index, cert) in certs.iter().enumerate() {
            let cert = as_bytes(cert, &format!("UDS cert `{signer_name}` entry {index}"))?;
            if cert.is_empty() {
                return Err(anyhow!(
                    "UDS cert `{signer_name}` entry {index} must not be empty"
                ));
            }
        }
    }

    Ok(())
}

fn parse_dice_entry_payload(value: &Value, label: &str) -> Result<CosePublicKeyInfo> {
    let payload_label = format!("{label} payload");
    let payload_entries = as_map(value, &payload_label)?;
    ensure_unique_integer_keys(payload_entries, &payload_label)?;

    if required_int_text(payload_entries, CWT_ISSUER, label, "issuer")?.is_empty() {
        return Err(anyhow!("{label} issuer must not be empty"));
    }
    if required_int_text(payload_entries, CWT_SUBJECT, label, "subject")?.is_empty() {
        return Err(anyhow!("{label} subject must not be empty"));
    }

    if let Some(profile_name) = map_get(payload_entries, DICE_PROFILE_NAME) {
        let profile_name = as_text(profile_name, &format!("{label} profile name"))?;
        if profile_name != ANDROID_DICE_PROFILE_VERSION {
            return Err(anyhow!(
                "{label} profile name must be `{ANDROID_DICE_PROFILE_VERSION}`, got `{profile_name}`"
            ));
        }
    }

    if required_int_bytes(payload_entries, DICE_KEY_USAGE, label, "key usage")?.is_empty() {
        return Err(anyhow!("{label} key usage must not be empty"));
    }

    for (field, field_label) in OPTIONAL_DICE_BYTE_FIELDS {
        if let Some(value) = map_get(payload_entries, field)
            && as_bytes(value, &format!("{label} {field_label}"))?.is_empty()
        {
            return Err(anyhow!(
                "{label} {field_label} must not be empty when present"
            ));
        }
    }

    if let Some(configuration) = map_get(payload_entries, DICE_CONFIGURATION_DESCRIPTOR) {
        let descriptor_label = format!("{label} configuration descriptor");
        let configuration = decode(as_bytes(configuration, &descriptor_label)?)?;
        as_map(&configuration, &descriptor_label)?;
    }

    let subject_key_bytes = required_int_bytes(
        payload_entries,
        DICE_SUBJECT_PUB_KEY,
        label,
        "subject public key",
    )?;
    let subject_key = decode(subject_key_bytes)?;
    parse_cose_public_key(
        as_map(&subject_key, "DICE chain subject COSE key")?,
        "DICE chain subject COSE key",
    )
}
