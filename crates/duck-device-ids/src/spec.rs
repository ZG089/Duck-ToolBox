use anyhow::Result;

use crate::{DeviceIdsProfile, error::DeviceIdsError};

pub(crate) const KM_TAG_ATTESTATION_ID_BRAND: u32 = 0x9000_02C6;
pub(crate) const KM_TAG_ATTESTATION_ID_DEVICE: u32 = 0x9000_02C7;
pub(crate) const KM_TAG_ATTESTATION_ID_PRODUCT: u32 = 0x9000_02C8;
pub(crate) const KM_TAG_ATTESTATION_ID_SERIAL: u32 = 0x9000_02C9;
pub(crate) const KM_TAG_ATTESTATION_ID_IMEI: u32 = 0x9000_02CA;
pub(crate) const KM_TAG_ATTESTATION_ID_MEID: u32 = 0x9000_02CB;
pub(crate) const KM_TAG_ATTESTATION_ID_MANUFACTURER: u32 = 0x9000_02CC;
pub(crate) const KM_TAG_ATTESTATION_ID_MODEL: u32 = 0x9000_02CD;
pub(crate) const KM_TAG_ATTESTATION_ID_SECOND_IMEI: u32 = 0x9000_02CE;

struct DeviceIdSpec {
    tag: u32,
    label: &'static str,
    field: &'static str,
    required: bool,
    value: fn(&DeviceIdsProfile) -> &str,
}

/// Order matches the reference provisioning tool; the TA expects this sequence.
/// Both MEID slots use the same tag because the TA has no dedicated second-MEID tag.
const DEVICE_ID_SPECS: &[DeviceIdSpec] = &[
    DeviceIdSpec {
        tag: KM_TAG_ATTESTATION_ID_BRAND,
        label: "BRAND",
        field: "brand",
        required: true,
        value: |p| &p.brand,
    },
    DeviceIdSpec {
        tag: KM_TAG_ATTESTATION_ID_DEVICE,
        label: "DEVICE",
        field: "device",
        required: true,
        value: |p| &p.device,
    },
    DeviceIdSpec {
        tag: KM_TAG_ATTESTATION_ID_PRODUCT,
        label: "PRODUCT",
        field: "product",
        required: true,
        value: |p| &p.product,
    },
    DeviceIdSpec {
        tag: KM_TAG_ATTESTATION_ID_SERIAL,
        label: "SERIAL",
        field: "serial",
        required: true,
        value: |p| &p.serial,
    },
    DeviceIdSpec {
        tag: KM_TAG_ATTESTATION_ID_IMEI,
        label: "IMEI",
        field: "imei",
        required: false,
        value: |p| &p.imei,
    },
    DeviceIdSpec {
        tag: KM_TAG_ATTESTATION_ID_SECOND_IMEI,
        label: "IMEI2",
        field: "imei2",
        required: false,
        value: |p| &p.imei2,
    },
    DeviceIdSpec {
        tag: KM_TAG_ATTESTATION_ID_MEID,
        label: "MEID",
        field: "meid",
        required: false,
        value: |p| &p.meid,
    },
    DeviceIdSpec {
        tag: KM_TAG_ATTESTATION_ID_MEID,
        label: "MEID2",
        field: "meid2",
        required: false,
        value: |p| &p.meid2,
    },
    DeviceIdSpec {
        tag: KM_TAG_ATTESTATION_ID_MANUFACTURER,
        label: "MANUFACTURER",
        field: "manufacturer",
        required: true,
        value: |p| &p.manufacturer,
    },
    DeviceIdSpec {
        tag: KM_TAG_ATTESTATION_ID_MODEL,
        label: "MODEL",
        field: "model",
        required: true,
        value: |p| &p.model,
    },
];

#[derive(Debug, Clone)]
pub(crate) struct SelectedId {
    pub(crate) tag: u32,
    pub(crate) label: &'static str,
    pub(crate) value: String,
}

/// Selects the IDs to provision, erroring when a required field is blank and skipping
/// blank optional ones.
pub(crate) fn collect_ids(profile: &DeviceIdsProfile) -> Result<Vec<SelectedId>> {
    let mut ids = Vec::new();

    for spec in DEVICE_ID_SPECS {
        let value = (spec.value)(profile).trim();
        if value.is_empty() {
            if spec.required {
                return Err(DeviceIdsError::MissingField(spec.field).into());
            }
            continue;
        }

        ids.push(SelectedId {
            tag: spec.tag,
            label: spec.label,
            value: value.to_owned(),
        });
    }

    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::{KM_TAG_ATTESTATION_ID_SECOND_IMEI, collect_ids};
    use crate::DeviceIdsProfile;

    fn profile() -> DeviceIdsProfile {
        DeviceIdsProfile {
            brand: "google".into(),
            device: "husky".into(),
            product: "husky".into(),
            serial: "ABC123".into(),
            manufacturer: "Google".into(),
            model: "Pixel".into(),
            ..DeviceIdsProfile::default()
        }
    }

    #[test]
    fn requires_base_fields() {
        let error = collect_ids(&DeviceIdsProfile::default()).unwrap_err();
        assert!(error.to_string().contains("brand"));
    }

    #[test]
    fn uses_second_imei_tag() {
        let mut profile = profile();
        profile.imei2 = "123456789012345".into();

        let ids = collect_ids(&profile).unwrap();
        let imei2 = ids.iter().find(|entry| entry.label == "IMEI2").unwrap();

        assert_eq!(imei2.tag, KM_TAG_ATTESTATION_ID_SECOND_IMEI);
    }

    #[test]
    fn matches_reference_order() {
        let mut profile = profile();
        profile.imei = "111111111111111".into();
        profile.imei2 = "222222222222222".into();
        profile.meid = "A0000000002321".into();
        profile.meid2 = "A0000000002322".into();

        let labels: Vec<_> = collect_ids(&profile)
            .unwrap()
            .iter()
            .map(|entry| entry.label)
            .collect();

        assert_eq!(
            labels,
            vec![
                "BRAND",
                "DEVICE",
                "PRODUCT",
                "SERIAL",
                "IMEI",
                "IMEI2",
                "MEID",
                "MEID2",
                "MANUFACTURER",
                "MODEL",
            ]
        );
    }
}
