//! Declarative decoding for custom keybox providers.
//!
//! Tricky Addon piped downloads through a user-supplied shell snippet guarded by a word
//! blocklist. Duck ToolBox never executes it: each `|`-separated step must be one of the
//! decoders below, which covers the provider formats Tricky Addon ships.

use base64ct::{Base64, Encoding};

use crate::error::TrickyError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Identity,
    Base64,
    Hex,
}

fn parse_step(step: &str) -> Result<Step, TrickyError> {
    let words: Vec<&str> = step.split_whitespace().collect();
    match words.as_slice() {
        [] | ["cat"] => Ok(Step::Identity),
        ["base64", "-d" | "--decode"] => Ok(Step::Base64),
        ["xxd", "-r", "-p"] | ["xxd", "-p", "-r"] | ["xxd", "-rp"] | ["xxd", "-pr"] => {
            Ok(Step::Hex)
        }
        _ => Err(TrickyError::UnsupportedDecoder(step.trim().to_owned())),
    }
}

/// Validates a pipeline without running it, so bad provider entries fail on save.
pub fn check(pipeline: &str) -> Result<(), TrickyError> {
    pipeline
        .split('|')
        .try_for_each(|step| parse_step(step).map(|_| ()))
}

pub fn apply(pipeline: &str, input: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut data = input.to_vec();
    for raw in pipeline.split('|') {
        data = match parse_step(raw)? {
            Step::Identity => data,
            Step::Base64 => {
                let compact: String = String::from_utf8_lossy(&data)
                    .chars()
                    .filter(|ch| !ch.is_whitespace())
                    .collect();
                Base64::decode_vec(&compact)
                    .map_err(|error| anyhow::anyhow!("base64 decode failed: {error}"))?
            }
            Step::Hex => {
                let compact: String = String::from_utf8_lossy(&data)
                    .chars()
                    .filter(char::is_ascii_hexdigit)
                    .collect();
                hex::decode(compact)
                    .map_err(|error| anyhow::anyhow!("hex decode failed: {error}"))?
            }
        };
    }
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::{apply, check};

    #[test]
    fn decodes_tricky_addon_default_pipeline() {
        use base64ct::Encoding;
        let xml = b"<AndroidAttestation/>";
        let hex = hex::encode(base64ct::Base64::encode_string(xml));
        let wrapped = format!("{}\n{}\n", &hex[..10], &hex[10..]);

        assert_eq!(
            apply("xxd -r -p | base64 -d", wrapped.as_bytes()).unwrap(),
            xml
        );
    }

    #[test]
    fn empty_and_cat_are_identity() {
        assert_eq!(apply("", b"abc").unwrap(), b"abc");
        assert_eq!(apply("cat", b"abc").unwrap(), b"abc");
    }

    #[test]
    fn rejects_arbitrary_commands() {
        assert!(check("rm -rf /").is_err());
        assert!(check("base64 -d | sh").is_err());
        assert!(check("xxd -r -p | base64 -d").is_ok());
    }
}
