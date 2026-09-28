use ciborium::{ser::into_writer, value::Value};
use p384::{SecretKey as P384SecretKey, elliptic_curve::sec1::ToEncodedPoint};

use super::{ALG_ES384, cose_key::parse_cose_public_key, verify_csr};
use crate::{
    cbor::{bytes, decode, encode, int},
    cose::{DeviceKeys, RPC_CURVE_25519, build_csr, generate_ec_keypair},
    profile::{DeviceInfo, DiceCurve},
};

fn valid_device_info() -> DeviceInfo {
    DeviceInfo {
        brand: "google".into(),
        model: "Pixel".into(),
        device: "pixel".into(),
        product: "pixel".into(),
        manufacturer: "Google".into(),
        fused: 1,
        vb_state: "green".into(),
        os_version: "13".into(),
        security_level: "tee".into(),
        bootloader_state: "locked".into(),
        boot_patch_level: 20260101,
        system_patch_level: 202601,
        vendor_patch_level: 20260101,
        vbmeta_digest: Some("11".repeat(32)),
        dice_issuer: "CN=Android".into(),
        dice_subject: "CN=Android".into(),
    }
}

fn csr_bytes(keys: &DeviceKeys, device: &DeviceInfo) -> Vec<u8> {
    let ec_key = generate_ec_keypair().unwrap();
    build_csr(
        keys,
        &[0x44; 32],
        &[ec_key.cose_public],
        &[0x55; 32],
        &[0x66; 8],
        device,
        RPC_CURVE_25519,
    )
    .unwrap()
    .csr_bytes
}

fn valid_csr() -> Vec<u8> {
    csr_bytes(&DeviceKeys::from_seed([0x11; 32]), &valid_device_info())
}

fn array(value: &mut Value) -> &mut Vec<Value> {
    match value {
        Value::Array(items) => items,
        other => panic!("expected CBOR array, got {other:?}"),
    }
}

fn byte_string(value: &mut Value) -> &mut Vec<u8> {
    match value {
        Value::Bytes(bytes) => bytes,
        other => panic!("expected CBOR bytes, got {other:?}"),
    }
}

fn encode_preserving_order(value: &Value) -> Vec<u8> {
    let mut bytes = Vec::new();
    into_writer(value, &mut bytes).unwrap();
    bytes
}

/// Decodes the signed payload inside the CSR, lets `edit` change it, and re-encodes the CSR.
fn tamper_signed_payload(csr: &[u8], edit: impl FnOnce(&mut Vec<Value>)) -> Vec<u8> {
    let mut decoded = decode(csr).unwrap();
    let signed_data = array(&mut array(&mut decoded)[3]);
    let payload_bytes = byte_string(&mut signed_data[2]);
    let mut signed_payload = decode(payload_bytes).unwrap();
    edit(array(&mut signed_payload));
    *payload_bytes = encode(&signed_payload).unwrap();
    encode(&decoded).unwrap()
}

/// Like [`tamper_signed_payload`] but edits the inner CSR payload array.
fn tamper_csr_payload(csr: &[u8], edit: impl FnOnce(&mut Vec<Value>), canonical: bool) -> Vec<u8> {
    tamper_signed_payload(csr, |signed_items| {
        let csr_payload_bytes = byte_string(&mut signed_items[1]);
        let mut csr_payload = decode(csr_payload_bytes).unwrap();
        edit(array(&mut csr_payload));
        *csr_payload_bytes = if canonical {
            encode(&csr_payload).unwrap()
        } else {
            encode_preserving_order(&csr_payload)
        };
    })
}

#[test]
fn verify_reports_valid_signature() {
    let report = verify_csr(&valid_csr()).unwrap();
    assert!(report.signature_valid);
    assert_eq!(report.csr_version, 3);
    assert_eq!(report.cert_type, "keymint");
}

#[test]
fn verify_rejects_wrong_authenticated_request_version() {
    let mut decoded = decode(&valid_csr()).unwrap();
    array(&mut decoded)[0] = int(9);

    let error = verify_csr(&encode(&decoded).unwrap()).unwrap_err();
    assert!(error.to_string().contains("CSR version"));
}

#[test]
fn verify_rejects_oversized_signed_challenge() {
    let tampered = tamper_signed_payload(&valid_csr(), |items| {
        items[0] = bytes(vec![0xAA; 65]);
    });

    let error = verify_csr(&tampered).unwrap_err();
    assert!(error.to_string().contains("challenge"));
}

#[test]
fn verify_rejects_noncanonical_device_info_order() {
    let tampered = tamper_csr_payload(
        &valid_csr(),
        |items| {
            let Value::Map(device_info) = &mut items[2] else {
                panic!("device info must decode as a map");
            };
            device_info.reverse();
        },
        false,
    );

    let error = verify_csr(&tampered).unwrap_err();
    assert!(error.to_string().contains("non-canonical"));
}

#[test]
fn verify_allows_missing_vbmeta_digest() {
    let mut device = valid_device_info();
    device.vbmeta_digest = None;

    let report = verify_csr(&csr_bytes(&DeviceKeys::from_seed([0x11; 32]), &device)).unwrap();
    assert!(report.signature_valid);
}

#[test]
fn verify_accepts_p256_dice_chain() {
    let keys = DeviceKeys::from_seed_with_curve([0x11; 32], DiceCurve::P256);
    let report = verify_csr(&csr_bytes(&keys, &valid_device_info())).unwrap();
    assert!(report.signature_valid);
}

#[test]
fn verify_rejects_invalid_keys_to_sign_shape() {
    let tampered = tamper_csr_payload(
        &valid_csr(),
        |items| {
            let Value::Map(entries) = &mut array(&mut items[3])[0] else {
                panic!("keysToSign entry must decode as a map");
            };
            entries.retain(
                |(key, _)| !matches!(key, Value::Integer(number) if i128::from(*number) == -3),
            );
        },
        true,
    );

    let error = verify_csr(&tampered).unwrap_err();
    assert!(error.to_string().contains("keysToSign entry 0"));
}

#[test]
fn verify_rejects_tampered_dice_chain_signature() {
    let mut decoded = decode(&valid_csr()).unwrap();
    let dice_chain = array(&mut array(&mut decoded)[2]);
    array(&mut dice_chain[1])[3] = bytes(vec![0; 64]);

    let error = verify_csr(&encode(&decoded).unwrap()).unwrap_err();
    assert!(error.to_string().contains("DICE chain entry 1 signature"));
}

#[test]
fn parse_cose_public_key_accepts_p384() {
    let secret_key = P384SecretKey::from_slice(&[0x11; 48]).unwrap();
    let encoded = secret_key.public_key().to_encoded_point(false);
    let x = encoded.x().unwrap().to_vec();
    let y = encoded.y().unwrap().to_vec();

    let entries = vec![
        (int(1), int(2)),
        (int(3), int(ALG_ES384)),
        (int(-1), int(2)),
        (int(-2), bytes(x.clone())),
        (int(-3), bytes(y.clone())),
    ];

    let parsed = parse_cose_public_key(&entries, "P-384 key").unwrap();
    assert_eq!(parsed.public_key_bytes.len(), x.len() + y.len());
}
