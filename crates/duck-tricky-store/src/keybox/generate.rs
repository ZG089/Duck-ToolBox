//! Self-signed "unknown" keybox, equivalent to Tricky Addon's WebCrypto generator.
//!
//! The certificates chain to nothing Google trusts, so attestation verdicts stay at
//! software level; the point is a keybox that is neither revoked nor shared. Keys are
//! generated with the shared aws-lc-rs backend and re-encoded to the SEC1/PKCS#1 PEM forms
//! that Tricky Store's keybox parser expects.

use anyhow::{Result, anyhow};
use pkcs8::{
    DecodePrivateKey, PrivateKeyInfoRef,
    der::{Decode, pem::LineEnding},
};
use rcgen::{
    CertificateParams, DistinguishedName, DnType, KeyPair, PKCS_ECDSA_P256_SHA256, PKCS_RSA_SHA256,
    RsaKeySize, SerialNumber,
};
use time::{Duration, OffsetDateTime};

const VALIDITY_DAYS: i64 = 3650;

pub fn unknown_keybox() -> Result<String> {
    let ec = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256)
        .map_err(|error| anyhow!("generate P-256 key: {error}"))?;
    let ec_pem = p256::SecretKey::from_pkcs8_der(ec.serialized_der())
        .map_err(|error| anyhow!("decode generated P-256 key: {error}"))?
        .to_sec1_pem(LineEnding::LF)
        .map_err(|error| anyhow!("encode SEC1 key: {error}"))?;

    let rsa = KeyPair::generate_rsa_for(&PKCS_RSA_SHA256, RsaKeySize::_2048)
        .map_err(|error| anyhow!("generate RSA-2048 key: {error}"))?;
    let rsa_pem = pkcs1_pem(rsa.serialized_der())?;

    Ok(render(&[
        ("ecdsa", ec_pem.as_str(), self_signed(&ec)?),
        ("rsa", rsa_pem.as_str(), self_signed(&rsa)?),
    ]))
}

/// A PKCS#8 `PrivateKeyInfo` for RSA wraps the PKCS#1 `RSAPrivateKey` DER verbatim
/// (RFC 5208 §5, RFC 8017 appendix A.1.2), so unwrapping it is enough.
fn pkcs1_pem(pkcs8_der: &[u8]) -> Result<String> {
    let info = PrivateKeyInfoRef::from_der(pkcs8_der)
        .map_err(|error| anyhow!("decode generated RSA key: {error}"))?;
    pkcs8::der::pem::encode_string(
        "RSA PRIVATE KEY",
        LineEnding::LF,
        info.private_key.as_bytes(),
    )
    .map_err(|error| anyhow!("encode PKCS#1 key: {error}"))
}

fn self_signed(key: &KeyPair) -> Result<String> {
    let mut params = CertificateParams::default();
    let mut name = DistinguishedName::new();
    name.push(DnType::CommonName, "Generated");
    params.distinguished_name = name;
    params.serial_number = Some(SerialNumber::from(1u64));
    let now = OffsetDateTime::now_utc();
    params.not_before = now;
    params.not_after = now + Duration::days(VALIDITY_DAYS);

    Ok(params
        .self_signed(key)
        .map_err(|error| anyhow!("self-sign certificate: {error}"))?
        .pem())
}

fn render(keys: &[(&str, &str, String)]) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<AndroidAttestation>\n    <NumberOfKeyboxes>1</NumberOfKeyboxes>\n    <Keybox DeviceID=\"sw\">\n",
    );
    for (algorithm, private_key, certificate) in keys {
        xml.push_str(&format!("        <Key algorithm=\"{algorithm}\">\n"));
        xml.push_str("            <PrivateKey format=\"pem\">\n");
        xml.push_str(&indent(private_key, 16));
        xml.push_str("            </PrivateKey>\n            <CertificateChain>\n");
        xml.push_str("                <NumberOfCertificates>1</NumberOfCertificates>\n");
        xml.push_str("                <Certificate format=\"pem\">\n");
        xml.push_str(&indent(certificate, 20));
        xml.push_str(
            "                </Certificate>\n            </CertificateChain>\n        </Key>\n",
        );
    }
    xml.push_str("    </Keybox>\n</AndroidAttestation>\n");
    xml
}

fn indent(block: &str, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    block
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| format!("{pad}{line}\n"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::unknown_keybox;
    use crate::keybox::validate::validate;

    #[test]
    fn generated_keybox_has_both_algorithms() {
        let xml = unknown_keybox().unwrap();
        let summary = validate(&xml).unwrap();

        assert!(summary.has_ecdsa && summary.has_rsa);
        assert_eq!(summary.chain_lengths, vec![1, 1]);
        assert!(xml.contains("-----BEGIN EC PRIVATE KEY-----"));
        assert!(xml.contains("-----BEGIN RSA PRIVATE KEY-----"));
    }
}
