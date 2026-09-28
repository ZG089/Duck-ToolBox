use anyhow::{Context, Result, anyhow};
use ciborium::value::Value;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use p256::ecdsa::{Signature as P256Signature, VerifyingKey as P256VerifyingKey};
use p384::ecdsa::{Signature as P384Signature, VerifyingKey as P384VerifyingKey};

use crate::{
    cbor::{as_bytes, as_i128, as_map, decode, map_get},
    cose::{ALG_EDDSA, ALG_ES256},
};

use super::{
    ALG_ES384, ED25519_CURVE, P256_COORD_LEN, P256_CURVE, P384_COORD_LEN, P384_CURVE,
    checks::{
        ensure_allowed_integer_keys, ensure_empty_map, ensure_unique_integer_keys, required_bytes,
        required_int,
    },
};

pub(super) struct CosePublicKeyInfo {
    pub(super) public_key_bytes: Vec<u8>,
    pub(super) algorithm: i128,
    verifier: CoseVerifier,
}

enum CoseVerifier {
    Ed25519(VerifyingKey),
    P256(P256VerifyingKey),
    P384(P384VerifyingKey),
}

pub(super) fn parse_cose_public_key(
    entries: &[(Value, Value)],
    label: &str,
) -> Result<CosePublicKeyInfo> {
    ensure_unique_integer_keys(entries, label)?;
    let key_type = required_int(entries, 1, label)?;
    let algorithm = required_int(entries, 3, label)?;
    let curve = required_int(entries, -1, label)?;

    match (key_type, algorithm, curve) {
        (1, ALG_EDDSA, ED25519_CURVE) => {
            ensure_allowed_integer_keys(entries, &[1, 3, -1, -2], label)?;
            let public_key = required_bytes(entries, -2, label, "public key")?.to_vec();
            let verifying_key = VerifyingKey::from_bytes(
                &<[u8; 32]>::try_from(public_key.as_slice())
                    .map_err(|_| anyhow!("{label} public key must be 32 bytes"))?,
            )
            .context("build Ed25519 verifying key")?;

            Ok(CosePublicKeyInfo {
                public_key_bytes: public_key,
                algorithm: ALG_EDDSA,
                verifier: CoseVerifier::Ed25519(verifying_key),
            })
        }
        (2, ALG_ES256, P256_CURVE) => {
            let (sec1, public_key) = ec2_point(entries, label, P256_COORD_LEN, "P-256")?;
            let verifying_key =
                P256VerifyingKey::from_sec1_bytes(&sec1).context("build ES256 verifying key")?;

            Ok(CosePublicKeyInfo {
                public_key_bytes: public_key,
                algorithm: ALG_ES256,
                verifier: CoseVerifier::P256(verifying_key),
            })
        }
        (2, ALG_ES384, P384_CURVE) => {
            let (sec1, public_key) = ec2_point(entries, label, P384_COORD_LEN, "P-384")?;
            let verifying_key =
                P384VerifyingKey::from_sec1_bytes(&sec1).context("build ES384 verifying key")?;

            Ok(CosePublicKeyInfo {
                public_key_bytes: public_key,
                algorithm: ALG_ES384,
                verifier: CoseVerifier::P384(verifying_key),
            })
        }
        _ => Err(anyhow!(
            "unsupported {label} parameters: kty={key_type}, alg={algorithm}, crv={curve}"
        )),
    }
}

/// Returns the uncompressed SEC1 point and the raw `x || y` bytes of an EC2 COSE key.
fn ec2_point(
    entries: &[(Value, Value)],
    label: &str,
    coord_len: usize,
    curve_name: &str,
) -> Result<(Vec<u8>, Vec<u8>)> {
    ensure_allowed_integer_keys(entries, &[1, 3, -1, -2, -3], label)?;
    let x = required_bytes(entries, -2, label, "x coordinate")?;
    let y = required_bytes(entries, -3, label, "y coordinate")?;
    if x.len() != coord_len || y.len() != coord_len {
        return Err(anyhow!(
            "{label} {curve_name} coordinates must each be {coord_len} bytes"
        ));
    }

    let raw = [x, y].concat();
    let sec1 = [&[0x04], raw.as_slice()].concat();
    Ok((sec1, raw))
}

pub(super) fn verify_signed_data_signature(
    key: &CosePublicKeyInfo,
    signature_bytes: &[u8],
    signature_input: &[u8],
) -> Result<bool> {
    Ok(match &key.verifier {
        CoseVerifier::Ed25519(verifying_key) => {
            let signature =
                Signature::from_slice(signature_bytes).context("parse Ed25519 signature")?;
            verifying_key.verify(signature_input, &signature).is_ok()
        }
        CoseVerifier::P256(verifying_key) => {
            let signature =
                P256Signature::from_slice(signature_bytes).context("parse ES256 signature")?;
            verifying_key.verify(signature_input, &signature).is_ok()
        }
        CoseVerifier::P384(verifying_key) => {
            let signature =
                P384Signature::from_slice(signature_bytes).context("parse ES384 signature")?;
            verifying_key.verify(signature_input, &signature).is_ok()
        }
    })
}

pub(super) struct ParsedCoseSign1<'a> {
    pub(super) protected: &'a [u8],
    pub(super) payload: &'a [u8],
    pub(super) signature: &'a [u8],
}

pub(super) fn parse_cose_sign1<'a>(
    sign1: &'a [Value],
    label: &str,
    signer: Option<&CosePublicKeyInfo>,
) -> Result<ParsedCoseSign1<'a>> {
    if sign1.len() != 4 {
        return Err(anyhow!("{label} must contain 4 array entries"));
    }

    let protected = as_bytes(&sign1[0], "COSE_Sign1 protected headers")?;
    ensure_empty_map(&sign1[1], &format!("{label} unprotected headers"))?;
    let protected_algorithm = parse_protected_algorithm(protected, label)?;
    if let Some(signer) = signer
        && signer.algorithm != protected_algorithm
    {
        return Err(anyhow!(
            "{label} protected algorithm {protected_algorithm} does not match signer algorithm {}",
            signer.algorithm
        ));
    }
    let payload = as_bytes(&sign1[2], "COSE_Sign1 payload")?;
    let signature = as_bytes(&sign1[3], "COSE_Sign1 signature")?;
    if signature.is_empty() {
        return Err(anyhow!("{label} signature must not be empty"));
    }

    Ok(ParsedCoseSign1 {
        protected,
        payload,
        signature,
    })
}

fn parse_protected_algorithm(protected: &[u8], label: &str) -> Result<i128> {
    let protected_value = decode(protected)?;
    let headers_label = format!("{label} protected headers");
    let headers = as_map(&protected_value, &headers_label)?;
    ensure_unique_integer_keys(headers, &headers_label)?;
    if headers.len() != 1 {
        return Err(anyhow!(
            "{label} protected headers must contain only the algorithm label"
        ));
    }

    let algorithm = as_i128(
        map_get(headers, 1).ok_or_else(|| anyhow!("{label} protected headers are missing `1`"))?,
        &format!("{label} algorithm"),
    )?;
    if matches!(algorithm, ALG_EDDSA | ALG_ES256 | ALG_ES384) {
        return Ok(algorithm);
    }

    Err(anyhow!(
        "{label} uses unsupported protected algorithm {algorithm}"
    ))
}
