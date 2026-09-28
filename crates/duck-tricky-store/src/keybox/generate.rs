//! The "unknown" keybox, Tricky Addon's WebCrypto generator done in the backend.
//!
//! Upstream writes one self-signed EC certificate and an RSA key without a chain, which only
//! Tricky Store loads: TEESimulator wants every chain to hold at least two certificates and
//! OhMyKeymint a chain for both keys. So each key here gets a batch certificate issued by a
//! generated root, like a factory keybox. Nothing chains to a Google root, so attestation
//! verdicts stay at software level; the point is a keybox that is neither revoked nor shared.
//! Keys use the shared aws-lc-rs backend and are re-encoded to the SEC1/PKCS#1 PEM forms that
//! the daemons' keybox parsers expect.

use anyhow::{Result, anyhow};
use pkcs8::{
    DecodePrivateKey, PrivateKeyInfoRef,
    der::{Decode, pem::LineEnding},
};
use rcgen::{
    BasicConstraints, CertificateParams, CertifiedIssuer, DistinguishedName, DnType, IsCa, KeyPair,
    KeyUsagePurpose, PKCS_ECDSA_P256_SHA256, PKCS_RSA_SHA256, RsaKeySize,
};
use time::{Duration, OffsetDateTime};

const VALIDITY_DAYS: i64 = 3650;

pub fn unknown_keybox() -> Result<String> {
    let ec = ec_key()?;
    let ec_pem = p256::SecretKey::from_pkcs8_der(ec.serialized_der())
        .map_err(|error| anyhow!("decode generated P-256 key: {error}"))?
        .to_sec1_pem(LineEnding::LF)
        .map_err(|error| anyhow!("encode SEC1 key: {error}"))?;
    let ec_chain = chain(&ec, ec_key()?)?;

    let rsa = rsa_key()?;
    let rsa_pem = pkcs1_pem(rsa.serialized_der())?;
    let rsa_chain = chain(&rsa, rsa_key()?)?;

    Ok(render(&[
        ("ecdsa", ec_pem.as_str(), ec_chain),
        ("rsa", rsa_pem.as_str(), rsa_chain),
    ]))
}

fn ec_key() -> Result<KeyPair> {
    KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256)
        .map_err(|error| anyhow!("generate P-256 key: {error}"))
}

fn rsa_key() -> Result<KeyPair> {
    KeyPair::generate_rsa_for(&PKCS_RSA_SHA256, RsaKeySize::_2048)
        .map_err(|error| anyhow!("generate RSA-2048 key: {error}"))
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

/// `[batch, root]` in PEM: the batch key signs attestation certificates, so it is a CA too.
fn chain(batch: &KeyPair, root_key: KeyPair) -> Result<Vec<String>> {
    let root = CertifiedIssuer::self_signed(
        ca_params("Generated Root", BasicConstraints::Unconstrained),
        root_key,
    )
    .map_err(|error| anyhow!("self-sign root certificate: {error}"))?;
    let leaf = ca_params("Generated", BasicConstraints::Constrained(0))
        .signed_by(batch, &root)
        .map_err(|error| anyhow!("sign batch certificate: {error}"))?;
    Ok(vec![leaf.pem(), root.pem()])
}

fn ca_params(common_name: &str, constraints: BasicConstraints) -> CertificateParams {
    let mut params = CertificateParams::default();
    let mut name = DistinguishedName::new();
    name.push(DnType::CommonName, common_name);
    params.distinguished_name = name;
    params.is_ca = IsCa::Ca(constraints);
    params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::DigitalSignature,
    ];
    let now = OffsetDateTime::now_utc();
    params.not_before = now;
    params.not_after = now + Duration::days(VALIDITY_DAYS);
    params
}

fn render(keys: &[(&str, &str, Vec<String>)]) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<AndroidAttestation>\n    <NumberOfKeyboxes>1</NumberOfKeyboxes>\n    <Keybox DeviceID=\"sw\">\n",
    );
    for (algorithm, private_key, chain) in keys {
        xml.push_str(&format!("        <Key algorithm=\"{algorithm}\">\n"));
        xml.push_str("            <PrivateKey format=\"pem\">\n");
        xml.push_str(&indent(private_key, 16));
        xml.push_str("            </PrivateKey>\n            <CertificateChain>\n");
        xml.push_str(&format!(
            "                <NumberOfCertificates>{}</NumberOfCertificates>\n",
            chain.len()
        ));
        for certificate in chain {
            xml.push_str("                <Certificate format=\"pem\">\n");
            xml.push_str(&indent(certificate, 20));
            xml.push_str("                </Certificate>\n");
        }
        xml.push_str("            </CertificateChain>\n        </Key>\n");
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
    use x509_cert::{
        Certificate,
        der::{DecodePem, Encode},
    };

    use super::unknown_keybox;
    use crate::keybox::validate::validate;

    #[test]
    fn generated_keybox_suits_every_daemon() {
        let xml = unknown_keybox().unwrap();
        let summary = validate(&xml).unwrap();

        assert!(summary.has_ecdsa && summary.has_rsa);
        assert_eq!(summary.chain_lengths, vec![2, 2]);
        assert!(summary.counts_declared_correctly);
        assert!(xml.contains("-----BEGIN EC PRIVATE KEY-----"));
        assert!(xml.contains("-----BEGIN RSA PRIVATE KEY-----"));
    }

    #[test]
    fn batch_certificates_are_issued_by_their_root() {
        let xml = unknown_keybox().unwrap();
        let pems: Vec<&str> = xml
            .split("-----BEGIN CERTIFICATE-----")
            .skip(1)
            .map(|rest| rest.split("-----END CERTIFICATE-----").next().unwrap())
            .collect();
        assert_eq!(pems.len(), 4);
        for pair in pems.chunks(2) {
            let parse = |body: &str| {
                let pem = format!("-----BEGIN CERTIFICATE-----{body}-----END CERTIFICATE-----");
                Certificate::from_pem(pem.trim().replace("\n                    ", "\n")).unwrap()
            };
            let (batch, root) = (parse(pair[0]), parse(pair[1]));
            assert_eq!(
                batch.tbs_certificate().issuer().to_der().unwrap(),
                root.tbs_certificate().subject().to_der().unwrap()
            );
            assert_eq!(
                root.tbs_certificate().issuer().to_der().unwrap(),
                root.tbs_certificate().subject().to_der().unwrap()
            );
        }
    }
}
