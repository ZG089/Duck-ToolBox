//! Provisions attestation device IDs (brand, model, IMEI, ...) into the Qualcomm Keymaster
//! trusted application through `libQSEEComAPI.so`.
//!
//! The tag values are the KeyMint `ATTESTATION_ID_*` tags from AOSP
//! `hardware/interfaces/security/keymint/aidl/android/hardware/security/keymint/Tag.aidl`.

use anyhow::{Result, bail};
use duck_core::AppPaths;
use duck_platform::props;
use serde::{Deserialize, Serialize};

mod cli;
mod command;
mod error;
#[allow(unsafe_code)]
mod qseecom;
mod report;
mod spec;

pub use cli::{Command, run};
pub use error::DeviceIdsError;

const DEFAULT_TA_NAME: &str = "keymaster64";
const DEFAULT_TA_PATH: &str = "/vendor/firmware_mnt/image";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceIdsProfile {
    #[serde(default)]
    pub brand: String,
    #[serde(default)]
    pub device: String,
    #[serde(default)]
    pub product: String,
    #[serde(default)]
    pub serial: String,
    #[serde(default)]
    pub manufacturer: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub imei: String,
    #[serde(default)]
    pub imei2: String,
    #[serde(default)]
    pub meid: String,
    #[serde(default)]
    pub meid2: String,
    #[serde(default = "default_ta_name")]
    pub ta_name: String,
    #[serde(default = "default_ta_path")]
    pub ta_path: String,
    #[serde(default)]
    pub dry_run: bool,
}

impl Default for DeviceIdsProfile {
    fn default() -> Self {
        Self {
            brand: String::new(),
            device: String::new(),
            product: String::new(),
            serial: String::new(),
            manufacturer: String::new(),
            model: String::new(),
            imei: String::new(),
            imei2: String::new(),
            meid: String::new(),
            meid2: String::new(),
            ta_name: default_ta_name(),
            ta_path: default_ta_path(),
            dry_run: false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProvisionedId {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceIdsProvisionResult {
    pub count: usize,
    pub ids: Vec<ProvisionedId>,
    pub dry_run: bool,
    pub ta_name: String,
    pub ta_path: String,
    pub loaded_library: Option<String>,
    pub ta_api_version: Option<String>,
    pub ta_version: Option<String>,
    pub report_path: String,
}

fn default_ta_name() -> String {
    DEFAULT_TA_NAME.into()
}

fn default_ta_path() -> String {
    DEFAULT_TA_PATH.into()
}

/// Reads the current identifiers from system properties.
pub fn detect_defaults() -> DeviceIdsProfile {
    DeviceIdsProfile {
        brand: props::first(&["ro.product.brand", "ro.product.vendor.brand"]),
        device: props::first(&["ro.product.device", "ro.product.vendor.device"]),
        product: props::first(&["ro.product.name", "ro.product.vendor.name"]),
        serial: props::first(&["ro.serialno", "ro.boot.serialno"]),
        manufacturer: props::first(&["ro.product.manufacturer", "ro.product.vendor.manufacturer"]),
        model: props::first(&["ro.product.model", "ro.product.vendor.model"]),
        imei: props::first(&[
            "persist.vendor.radio.imei",
            "persist.radio.imei",
            "vendor.ril.imei",
            "ril.gsm.imei",
            "ro.ril.oem.imei",
            "ro.ril.oem.imei1",
        ]),
        imei2: props::first(&[
            "persist.vendor.radio.imei2",
            "persist.radio.imei2",
            "vendor.ril.imei2",
            "ril.gsm.imei2",
            "ro.ril.oem.imei2",
        ]),
        meid: props::first(&[
            "persist.vendor.radio.meid",
            "persist.radio.meid",
            "vendor.ril.meid",
            "ro.ril.oem.meid",
        ]),
        meid2: props::first(&[
            "persist.vendor.radio.meid2",
            "persist.radio.meid2",
            "vendor.ril.meid2",
            "ro.ril.oem.meid2",
        ]),
        ..DeviceIdsProfile::default()
    }
}

pub fn provision(paths: &AppPaths, profile: DeviceIdsProfile) -> Result<DeviceIdsProvisionResult> {
    let resolved = merge_missing_from_system(profile);
    let ids = spec::collect_ids(&resolved)?;
    let command = command::build(&ids)?;

    let mut session_info = qseecom::SessionInfo::default();
    if !resolved.dry_run {
        if !cfg!(target_os = "android") {
            bail!("device ID provisioning is only supported when Duck ToolBox runs on Android");
        }
        session_info = qseecom::provision(&resolved.ta_path, &resolved.ta_name, &command)?;
    }

    let provisioned = ids
        .iter()
        .map(|entry| ProvisionedId {
            label: entry.label.into(),
            value: entry.value.clone(),
        })
        .collect::<Vec<_>>();
    let report_path = report::write(paths, &resolved, &provisioned, &session_info, &command)?;

    Ok(DeviceIdsProvisionResult {
        count: provisioned.len(),
        ids: provisioned,
        dry_run: resolved.dry_run,
        ta_name: resolved.ta_name,
        ta_path: resolved.ta_path,
        loaded_library: session_info.loaded_library,
        ta_api_version: session_info.ta_api_version,
        ta_version: session_info.ta_version,
        report_path,
    })
}

fn merge_missing_from_system(mut profile: DeviceIdsProfile) -> DeviceIdsProfile {
    let detected = detect_defaults();
    for (target, fallback) in [
        (&mut profile.brand, detected.brand),
        (&mut profile.device, detected.device),
        (&mut profile.product, detected.product),
        (&mut profile.serial, detected.serial),
        (&mut profile.manufacturer, detected.manufacturer),
        (&mut profile.model, detected.model),
        (&mut profile.imei, detected.imei),
        (&mut profile.imei2, detected.imei2),
        (&mut profile.meid, detected.meid),
        (&mut profile.meid2, detected.meid2),
        (&mut profile.ta_name, detected.ta_name),
        (&mut profile.ta_path, detected.ta_path),
    ] {
        if target.trim().is_empty() {
            *target = fallback;
        }
    }
    profile
}

#[cfg(test)]
mod tests {
    use super::detect_defaults;

    #[test]
    fn detect_defaults_keeps_ta_defaults() {
        let detected = detect_defaults();

        assert_eq!(detected.ta_name, "keymaster64");
        assert_eq!(detected.ta_path, "/vendor/firmware_mnt/image");
    }
}
