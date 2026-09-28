//! Offline verification of an `AuthenticatedRequest` CSR.
//!
//! Mirrors the checks in AOSP's `hardware/interfaces/security/rkp/aidl/.../
//! generateCertificateRequestV2.cddl` and the reference validator used by
//! `rkp_factory_extraction_tool`.

use anyhow::{Result, anyhow};
use serde::Serialize;

use crate::{
    cbor::{as_array, as_bytes, as_i128, as_text, decode},
    cose::build_sig_structure,
};

mod checks;
mod cose_key;
mod device_info;
mod dice;
mod keys_to_sign;
#[cfg(test)]
mod tests;

use checks::{ensure_array_len, validate_challenge};
use cose_key::{parse_cose_sign1, verify_signed_data_signature};
use device_info::validate_device_info;
use dice::{extract_uds_key, validate_uds_certs, verify_dice_chain_and_extract_leaf_key};
use keys_to_sign::validate_keys_to_sign;

const AUTHENTICATED_REQUEST_VERSION: i128 = 1;
const CSR_PAYLOAD_VERSION: i128 = 3;
const ALG_ES384: i128 = -35;
const ED25519_CURVE: i128 = 6;
const P256_CURVE: i128 = 1;
const P384_CURVE: i128 = 2;
const P256_COORD_LEN: usize = 32;
const P384_COORD_LEN: usize = 48;
const MAX_CHALLENGE_LEN: usize = 64;
const TEST_KEY_LABEL: i128 = -70000;
const DICE_PROFILE_NAME: i128 = -4_670_554;
const ANDROID_DICE_PROFILE_VERSION: &str = "android.15";

#[derive(Debug, Clone, Serialize)]
pub struct VerifyReport {
    pub version: i128,
    pub dice_entries: usize,
    pub uds_pub_hex: String,
    pub signature_valid: bool,
    pub csr_version: i128,
    pub cert_type: String,
    pub brand: Option<String>,
    pub keys_to_sign: usize,
}

pub fn verify_csr(bytes: &[u8]) -> Result<VerifyReport> {
    let value = decode(bytes)?;
    let csr = as_array(&value, "CSR")?;
    ensure_array_len(csr, 4, "CSR")?;

    let version = as_i128(&csr[0], "CSR version")?;
    if version != AUTHENTICATED_REQUEST_VERSION {
        return Err(anyhow!(
            "CSR version must be {AUTHENTICATED_REQUEST_VERSION}, got {version}"
        ));
    }

    validate_uds_certs(&csr[1])?;

    let dice_chain = as_array(&csr[2], "DICE chain")?;
    if dice_chain.len() < 2 {
        return Err(anyhow!(
            "DICE chain must contain a UDS key and at least one DICE entry"
        ));
    }
    let uds = extract_uds_key(dice_chain)?;
    let leaf = verify_dice_chain_and_extract_leaf_key(dice_chain)?;

    let signed_data = parse_cose_sign1(
        as_array(&csr[3], "signed data")?,
        "signed data",
        Some(&leaf),
    )?;
    let signature_input =
        build_sig_structure("Signature1", signed_data.protected, signed_data.payload)?;
    let signature_valid =
        verify_signed_data_signature(&leaf, signed_data.signature, &signature_input)?;

    let signed_payload = decode(signed_data.payload)?;
    let signed_items = as_array(&signed_payload, "signed payload")?;
    ensure_array_len(signed_items, 2, "signed payload")?;
    validate_challenge(as_bytes(&signed_items[0], "challenge")?, "challenge")?;

    let csr_payload = decode(as_bytes(&signed_items[1], "CSR payload bytes")?)?;
    let csr_payload_items = as_array(&csr_payload, "CSR payload")?;
    ensure_array_len(csr_payload_items, 4, "CSR payload")?;

    let csr_version = as_i128(&csr_payload_items[0], "CSR payload version")?;
    if csr_version != CSR_PAYLOAD_VERSION {
        return Err(anyhow!(
            "CSR payload version must be {CSR_PAYLOAD_VERSION}, got {csr_version}"
        ));
    }

    let cert_type = as_text(&csr_payload_items[1], "cert type")?.to_owned();
    if cert_type.trim().is_empty() {
        return Err(anyhow!("cert type must not be empty"));
    }

    let brand = validate_device_info(&csr_payload_items[2])?;
    let keys_to_sign = as_array(&csr_payload_items[3], "keysToSign")?;
    validate_keys_to_sign(keys_to_sign)?;

    Ok(VerifyReport {
        version,
        dice_entries: dice_chain.len(),
        uds_pub_hex: hex::encode(&uds.public_key_bytes),
        signature_valid,
        csr_version,
        cert_type,
        brand: Some(brand),
        keys_to_sign: keys_to_sign.len(),
    })
}
