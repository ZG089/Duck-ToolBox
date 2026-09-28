//! Profile defaults read from the running device's build properties: the values its own
//! KeyMint reports in `DeviceInfo` for a factory RKP request.

use std::collections::BTreeMap;

use super::model::{DEFAULT_SERVER_URL, ProfileData};

/// Reads every property once and derives a profile from it. Nothing is saved.
pub fn detect() -> ProfileData {
    from_props(&duck_platform::props::all())
}

pub fn from_props(props: &BTreeMap<String, String>) -> ProfileData {
    let get = |key: &str| {
        props
            .get(key)
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
    };
    let first = |keys: &[&str]| keys.iter().find_map(|key| get(key));

    let mut profile = ProfileData::default();
    let device = &mut profile.device;
    let set = |target: &mut String, key: &str| {
        if let Some(value) = get(key) {
            *target = value.to_owned();
        }
    };
    set(&mut device.brand, "ro.product.brand");
    set(&mut device.model, "ro.product.model");
    set(&mut device.device, "ro.product.device");
    set(&mut device.product, "ro.product.name");
    set(&mut device.manufacturer, "ro.product.manufacturer");
    set(&mut device.os_version, "ro.build.version.release");
    set(&mut device.vb_state, "ro.boot.verifiedbootstate");
    set(&mut profile.fingerprint.value, "ro.build.fingerprint");

    if let Some(state) = get("ro.boot.vbmeta.device_state") {
        device.bootloader_state = state.to_owned();
    } else if let Some(locked) = get("ro.boot.flash.locked") {
        device.bootloader_state = if locked == "1" { "locked" } else { "unlocked" }.into();
    }

    let system_patch = get("ro.build.version.security_patch");
    let vendor_patch = first(&["ro.vendor.build.security_patch"]).or(system_patch);
    // KeyMint's boot patch level comes from the boot image (ro.vendor.boot_security_patch).
    let boot_patch = first(&["ro.vendor.boot_security_patch"]).or(vendor_patch);
    if let Some(level) = system_patch.and_then(|value| patch_level(value, 6)) {
        device.system_patch_level = level;
    }
    if let Some(level) = vendor_patch.and_then(|value| patch_level(value, 8)) {
        device.vendor_patch_level = level;
    }
    if let Some(level) = boot_patch.and_then(|value| patch_level(value, 8)) {
        device.boot_patch_level = level;
    }

    device.vbmeta_digest = get("ro.boot.vbmeta.digest").map(str::to_owned);

    if let Some(host) = get("remote_provisioning.hostname") {
        let host = host
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/')
            .trim_end_matches("/v1");
        profile.server_url = format!("https://{host}/v1");
    } else {
        profile.server_url = DEFAULT_SERVER_URL.into();
    }

    profile
}

/// `2026-09-05` as `20260905` (8 digits) or `202609` (6 digits).
fn patch_level(value: &str, digits: usize) -> Option<u32> {
    let compact: String = value.chars().filter(char::is_ascii_digit).collect();
    compact.get(..digits)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::from_props;

    #[test]
    fn derives_device_info_from_build_props() {
        let props: BTreeMap<String, String> = [
            ("ro.product.brand", "google"),
            ("ro.product.model", "Pixel 9"),
            ("ro.product.device", "tokay"),
            ("ro.product.name", "tokay"),
            ("ro.product.manufacturer", "Google"),
            ("ro.build.version.release", "16"),
            ("ro.boot.verifiedbootstate", "green"),
            ("ro.boot.flash.locked", "1"),
            ("ro.build.version.security_patch", "2026-09-05"),
            ("ro.vendor.build.security_patch", "2026-08-05"),
            ("ro.boot.vbmeta.digest", "ab"),
            ("remote_provisioning.hostname", "remoteprovisioning.googleapis.com"),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect();

        let profile = from_props(&props);
        assert_eq!(profile.device.model, "Pixel 9");
        assert_eq!(profile.device.bootloader_state, "locked");
        assert_eq!(profile.device.system_patch_level, 202609);
        assert_eq!(profile.device.vendor_patch_level, 20260805);
        assert_eq!(profile.device.boot_patch_level, 20260805);
        assert_eq!(profile.device.vbmeta_digest.as_deref(), Some("ab"));
        assert_eq!(
            profile.server_url,
            "https://remoteprovisioning.googleapis.com/v1"
        );
    }

    #[test]
    fn keeps_generic_defaults_without_props() {
        let profile = from_props(&BTreeMap::new());
        assert_eq!(profile.device.brand, "generic");
        assert!(profile.device.vbmeta_digest.is_none());
    }
}
