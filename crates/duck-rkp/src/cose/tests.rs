use ciborium::value::Value;
use p256::elliptic_curve::sec1::ToEncodedPoint;

use super::{
    ALG_ES256, DeviceKeys, RPC_CURVE_25519, RPC_CURVE_P256, build_csr, build_sig_structure,
    csr::build_dice_chain, device_info_to_cbor, generate_ec_keypair,
};
use crate::{
    cbor::{decode, encode, map_get_text},
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
        vbmeta_digest: Some("22".repeat(32)),
        dice_issuer: "CN=Android".into(),
        dice_subject: "CN=Android".into(),
    }
}

fn map_entries(value: Value) -> Vec<(Value, Value)> {
    match value {
        Value::Map(entries) => entries,
        other => panic!("expected a CBOR map, got {other:?}"),
    }
}

fn has_text_key(entries: &[(Value, Value)], key: &str) -> bool {
    entries
        .iter()
        .any(|(candidate, _)| matches!(candidate, Value::Text(text) if text == key))
}

#[test]
fn generated_p256_key_is_valid_cose_key() {
    let key = generate_ec_keypair().unwrap();
    assert_eq!(map_entries(key.cose_public).len(), 5);
}

#[test]
fn device_info_cbor_includes_expected_fields() {
    let value = device_info_to_cbor(&valid_device_info()).unwrap();
    assert!(!encode(&value).unwrap().is_empty());

    let entries = map_entries(value);
    assert!(map_get_text(&entries, "vbmeta_digest").is_some());
    assert!(map_get_text(&entries, "version").is_none());
}

#[test]
fn device_info_cbor_omits_blank_os_version_for_strongbox() {
    let mut device = valid_device_info();
    device.security_level = "strongbox".into();
    device.os_version.clear();

    let entries = map_entries(device_info_to_cbor(&device).unwrap());
    assert!(!has_text_key(&entries, "os_version"));
}

#[test]
fn device_info_cbor_omits_missing_vbmeta_digest() {
    let mut device = valid_device_info();
    device.vbmeta_digest = None;

    let entries = map_entries(device_info_to_cbor(&device).unwrap());
    assert!(!has_text_key(&entries, "vbmeta_digest"));
}

#[test]
fn built_csr_produces_bytes() {
    let keys = DeviceKeys::from_seed([0x11; 32]);
    let ec = generate_ec_keypair().unwrap();
    let bundle = build_csr(
        &keys,
        &[0x22; 32],
        &[ec.cose_public],
        &[0x33; 32],
        &[0x44; 8],
        &valid_device_info(),
        RPC_CURVE_25519,
    )
    .unwrap();

    assert!(!bundle.csr_bytes.is_empty());
    assert!(bundle.protected_data_len > 0);
}

#[test]
fn device_keys_support_p256_curve() {
    let keys = DeviceKeys::from_seed_with_curve([0x11; 32], DiceCurve::P256);

    assert_eq!(keys.curve(), DiceCurve::P256);
    assert_eq!(keys.algorithm(), ALG_ES256);
    assert_eq!(keys.public_key_bytes().len(), 64);
    assert!(!keys.sign(b"duck").is_empty());
}

#[test]
fn build_csr_supports_p256_eek() {
    let keys = DeviceKeys::from_seed([0x11; 32]);
    let ec = generate_ec_keypair().unwrap();
    let server = generate_ec_keypair().unwrap();
    let encoded = server.secret_key.public_key().to_encoded_point(false);
    let server_pub = [
        encoded.x().unwrap().as_slice(),
        encoded.y().unwrap().as_slice(),
    ]
    .concat();

    let bundle = build_csr(
        &keys,
        &[0x22; 32],
        &[ec.cose_public],
        &server_pub,
        &[0x55; 8],
        &valid_device_info(),
        RPC_CURVE_P256,
    )
    .unwrap();

    assert!(!bundle.csr_bytes.is_empty());
    assert!(bundle.protected_data_len > 0);
}

#[test]
fn build_csr_rejects_unknown_eek_curve() {
    let keys = DeviceKeys::from_seed([0x11; 32]);
    let ec = generate_ec_keypair().unwrap();

    let error = build_csr(
        &keys,
        &[0x22; 32],
        &[ec.cose_public],
        &[0x33; 32],
        &[0x44; 8],
        &valid_device_info(),
        99,
    )
    .unwrap_err();

    assert!(error.to_string().contains("unsupported EEK curve"));
}

#[test]
fn signature_structure_matches_cose_shape() {
    let encoded = build_sig_structure("Signature1", &[1, 2], &[3, 4]).unwrap();
    assert!(!encoded.is_empty());
}

#[test]
fn dice_entry_payload_omits_profile_name_like_reference_tool() {
    let keys = DeviceKeys::from_seed([0x11; 32]);
    let Value::Array(chain) = build_dice_chain(&keys, &valid_device_info()).unwrap() else {
        panic!("DICE chain must be an array");
    };
    let Value::Array(entry) = &chain[1] else {
        panic!("DICE entry must be a COSE_Sign1 array");
    };
    let Value::Bytes(payload) = &entry[2] else {
        panic!("DICE payload must be wrapped in bytes");
    };

    let entries = map_entries(decode(payload).unwrap());
    assert!(entries.iter().all(
        |(key, _)| !matches!(key, Value::Integer(number) if i128::from(*number) == -4_670_554)
    ));
}
