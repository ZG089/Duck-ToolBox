//! `keybox.xml` validation and clean-up.

use std::io::Cursor;

use anyhow::{Context, Result};
use quick_xml::{Reader, Writer, XmlVersion, events::Event, name::QName};
use serde::Serialize;

use crate::error::TrickyError;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct KeyboxSummary {
    pub keyboxes: usize,
    pub has_ecdsa: bool,
    pub has_rsa: bool,
    /// Certificate count of each `<Key>`, in document order; 0 when a key has no chain.
    pub chain_lengths: Vec<usize>,
    /// Every `<NumberOfCertificates>` matches the certificates that follow it.
    #[serde(skip)]
    pub counts_declared_correctly: bool,
}

/// Removes comments, a UTF-8 BOM and zero-width characters. Watermarks hidden in comments
/// or invisible characters make OhMyKeymint and TEESimulator reject otherwise valid files.
pub fn clean(xml: &str) -> Result<String> {
    let stripped: String = xml
        .trim_start_matches('\u{feff}')
        .chars()
        .filter(|ch| !matches!(ch, '\u{200b}'..='\u{200d}' | '\u{2060}' | '\u{feff}'))
        .collect();

    let mut reader = Reader::from_str(stripped.trim());
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    loop {
        match reader.read_event().context("parse keybox XML")? {
            Event::Eof => break,
            Event::Comment(_) => {}
            event => writer.write_event(event).context("rewrite keybox XML")?,
        }
    }

    let mut cleaned =
        String::from_utf8(writer.into_inner().into_inner()).context("keybox XML is not UTF-8")?;
    cleaned.push('\n');
    Ok(cleaned)
}

/// Checks the structure Tricky Store documents: one `AndroidAttestation` root holding
/// `Keybox` entries whose `Key`s carry a PEM `PrivateKey` and a `CertificateChain`.
pub fn validate(xml: &str) -> Result<KeyboxSummary> {
    let invalid = |reason: &str| anyhow::Error::from(TrickyError::InvalidKeybox(reason.into()));
    let trimmed = xml.trim();
    if trimmed.is_empty() {
        return Err(invalid("file is empty"));
    }

    let mut reader = Reader::from_str(trimmed);
    reader.config_mut().trim_text(true);

    let mut summary = KeyboxSummary {
        counts_declared_correctly: true,
        ..KeyboxSummary::default()
    };
    let mut stack: Vec<String> = Vec::new();
    let mut root_closed = false;
    let mut key_has_private = false;
    let mut key_certificates = 0usize;
    let mut declared: Option<usize> = None;

    loop {
        let event = reader.read_event().map_err(|error| {
            invalid(&format!(
                "XML error near byte {}: {error}",
                reader.buffer_position()
            ))
        })?;
        match event {
            Event::Start(ref tag) | Event::Empty(ref tag) => {
                let name = local_name(tag.name());
                let is_empty = matches!(event, Event::Empty(_));
                if stack.is_empty() {
                    if root_closed || name != "AndroidAttestation" {
                        return Err(invalid("root element must be a single AndroidAttestation"));
                    }
                } else if stack.len() == 1 && name == "Keybox" {
                    summary.keyboxes += 1;
                }

                match name.as_str() {
                    "Key" => {
                        key_has_private = false;
                        key_certificates = 0;
                        let algorithm = tag
                            .try_get_attribute("algorithm")
                            .ok()
                            .flatten()
                            .and_then(|attr| attr.normalized_value(XmlVersion::Implicit1_0).ok())
                            .map(|value| value.trim().to_ascii_lowercase());
                        match algorithm.as_deref() {
                            Some("ecdsa" | "ec") => summary.has_ecdsa = true,
                            Some("rsa") => summary.has_rsa = true,
                            _ => {
                                return Err(invalid(
                                    "every Key needs algorithm=\"ecdsa\" or \"rsa\"",
                                ));
                            }
                        }
                    }
                    "PrivateKey" => key_has_private = true,
                    "CertificateChain" => declared = None,
                    "Certificate" => key_certificates += 1,
                    _ => {}
                }

                if !is_empty {
                    stack.push(name);
                } else if stack.is_empty() {
                    root_closed = true;
                }
            }
            Event::End(tag) => {
                let name = local_name(tag.name());
                if stack.pop().as_deref() != Some(name.as_str()) {
                    return Err(invalid("mismatched closing tag"));
                }
                match name.as_str() {
                    "CertificateChain" => {
                        if declared
                            .take()
                            .is_some_and(|count| count != key_certificates)
                        {
                            summary.counts_declared_correctly = false;
                        }
                    }
                    "Key" if !key_has_private => {
                        return Err(invalid("a Key is missing its PrivateKey"));
                    }
                    "Key" => summary.chain_lengths.push(key_certificates),
                    _ => {}
                }
                if stack.is_empty() {
                    root_closed = true;
                }
            }
            Event::Text(ref text)
                if stack
                    .last()
                    .is_some_and(|tag| tag == "NumberOfCertificates") =>
            {
                declared = text
                    .xml_content(XmlVersion::Implicit1_0)
                    .trim()
                    .parse()
                    .ok();
            }
            Event::Eof => break,
            _ => {}
        }
    }

    if !root_closed {
        return Err(invalid("AndroidAttestation is not closed"));
    }
    if summary.keyboxes == 0 {
        return Err(invalid("no Keybox entry found"));
    }
    if !summary.has_ecdsa && !summary.has_rsa {
        return Err(invalid("no ecdsa or rsa Key found"));
    }
    // Keys without a chain are allowed (Tricky Addon's generator writes an RSA key without
    // one); each daemon's own rules are checked when installing.
    if summary.chain_lengths.iter().all(|count| *count == 0) {
        return Err(invalid("no Key has a certificate"));
    }

    Ok(summary)
}

fn local_name(name: QName<'_>) -> String {
    name.local_name().as_ref().to_owned()
}

#[cfg(test)]
mod tests {
    use super::{clean, validate};

    const MINIMAL: &str = r#"<?xml version="1.0"?>
<AndroidAttestation>
  <NumberOfKeyboxes>1</NumberOfKeyboxes>
  <Keybox DeviceID="x">
    <Key algorithm="ecdsa">
      <PrivateKey format="pem">k</PrivateKey>
      <CertificateChain><NumberOfCertificates>1</NumberOfCertificates><Certificate format="pem">c</Certificate></CertificateChain>
    </Key>
  </Keybox>
</AndroidAttestation>"#;

    #[test]
    fn accepts_minimal_keybox() {
        let summary = validate(MINIMAL).unwrap();
        assert_eq!(summary.keyboxes, 1);
        assert!(summary.has_ecdsa);
        assert!(!summary.has_rsa);
        assert_eq!(summary.chain_lengths, vec![1]);
    }

    #[test]
    fn accepts_bundled_aosp_keybox() {
        let summary = validate(include_str!("../../assets/aosp-keybox.xml")).unwrap();
        assert!(summary.has_ecdsa && summary.has_rsa);
        assert_eq!(summary.chain_lengths, vec![2, 2]);
    }

    #[test]
    fn rejects_wrong_root_and_text_markers() {
        assert!(validate(r#"<Keybox DeviceID="x"></Keybox>"#).is_err());
        assert!(
            validate(r#"<AndroidAttestation><Note>&lt;Keybox&gt;</Note></AndroidAttestation>"#)
                .is_err()
        );
        assert!(validate("").is_err());
    }

    #[test]
    fn counts_certificates_per_key_including_keys_without_a_chain() {
        let rsa = r#"<Key algorithm="rsa"><PrivateKey format="pem">r</PrivateKey></Key>
  </Keybox>"#;
        let summary = validate(&MINIMAL.replace("  </Keybox>", rsa)).unwrap();
        assert!(summary.has_rsa);
        assert_eq!(summary.chain_lengths, vec![1, 0]);
        assert!(summary.counts_declared_correctly);
    }

    #[test]
    fn notices_a_wrong_certificate_count() {
        let wrong = MINIMAL.replace(
            "<NumberOfCertificates>1</NumberOfCertificates>",
            "<NumberOfCertificates>2</NumberOfCertificates>",
        );
        assert!(!validate(&wrong).unwrap().counts_declared_correctly);
    }

    #[test]
    fn rejects_key_without_private_key() {
        let broken = MINIMAL.replace(r#"<PrivateKey format="pem">k</PrivateKey>"#, "");
        assert!(validate(&broken).is_err());
    }

    #[test]
    fn clean_strips_comments_and_invisible_characters() {
        let dirty = format!(
            "\u{feff}<!-- watermark -->{}",
            MINIMAL.replace("<Keybox", "<Key\u{200b}box")
        );
        let cleaned = clean(&dirty).unwrap();

        assert!(!cleaned.contains("watermark"));
        assert!(!cleaned.contains('\u{200b}'));
        assert!(!cleaned.starts_with('\u{feff}'));
        validate(&cleaned).unwrap();
    }
}
