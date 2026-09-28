use anyhow::{Result, bail};
use ciborium::{ser::into_writer, value::Value};

use crate::spec::SelectedId;

const CMD_PROVISION_DEVICE_IDS: u32 = 0x220A;

/// Builds the `PROVISION_DEVICE_IDS` command: a little-endian operation id followed by a
/// CBOR map of `{ 22: count, <tag>: <value>, ... }`. Map entries keep insertion order
/// because the trusted application reads them positionally.
pub(crate) fn build(ids: &[SelectedId]) -> Result<Vec<u8>> {
    if ids.is_empty() {
        bail!("device ID provisioning requires at least one ID");
    }

    let mut entries = Vec::with_capacity(ids.len() + 1);
    entries.push((uint(22), uint(ids.len() as u128)));
    for id in ids {
        entries.push((tag_key(id.tag), Value::Bytes(id.value.as_bytes().to_vec())));
    }

    let mut payload = Vec::new();
    into_writer(&Value::Map(entries), &mut payload)
        .map_err(|error| anyhow::anyhow!("encode device ID CBOR payload: {error}"))?;

    let mut command = CMD_PROVISION_DEVICE_IDS.to_le_bytes().to_vec();
    command.extend_from_slice(&payload);
    Ok(command)
}

fn uint(value: u128) -> Value {
    Value::Integer(
        value
            .try_into()
            .expect("device ID counts fit in a CBOR integer"),
    )
}

/// KeyMint tags are `u32`s reinterpreted as signed CBOR integers, so tags with the high
/// bit set become negative keys.
fn tag_key(tag: u32) -> Value {
    Value::Integer(
        i128::from(tag as i32)
            .try_into()
            .expect("i32 fits in a CBOR integer"),
    )
}

#[cfg(test)]
mod tests {
    use super::build;
    use crate::DeviceIdsProfile;
    use crate::spec::collect_ids;

    #[test]
    fn build_matches_reference_bytes() {
        let profile = DeviceIdsProfile {
            brand: "google".into(),
            device: "husky".into(),
            product: "husky".into(),
            serial: "ABC123".into(),
            imei: "111111111111111".into(),
            imei2: "222222222222222".into(),
            manufacturer: "Google".into(),
            model: "Pixel".into(),
            ..DeviceIdsProfile::default()
        };

        let command = build(&collect_ids(&profile).unwrap()).unwrap();

        // Captured from the original hand-rolled CBOR encoder to lock the wire format.
        let expected = "0a220000a916083a6ffffd3946676f6f676c653a6ffffd38456875736b793a6ffffd374568\
                        75736b793a6ffffd36464142433132333a6ffffd354f3131313131313131313131313131313a6\
                        ffffd314f3232323232323232323232323232323a6ffffd3346476f6f676c653a6ffffd3245506\
                        978656c";
        assert_eq!(hex::encode(&command), expected);
    }

    #[test]
    fn build_starts_with_operation_id() {
        let profile = DeviceIdsProfile {
            brand: "google".into(),
            device: "husky".into(),
            product: "husky".into(),
            serial: "ABC123".into(),
            manufacturer: "Google".into(),
            model: "Pixel".into(),
            ..DeviceIdsProfile::default()
        };

        let command = build(&collect_ids(&profile).unwrap()).unwrap();
        assert_eq!(&command[..4], &0x220Au32.to_le_bytes());
    }
}
