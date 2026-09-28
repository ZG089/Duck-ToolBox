use std::collections::BTreeSet;

use anyhow::{Context, Result};
use duck_core::{Sysroot, fs::write_bytes_preserving};
use serde_json::{Map, Value, json};

use crate::{
    error::TrickyError,
    model::{Backend, ConfigData, Policy, PolicyField, PolicySchema, TargetEntry, TargetMode},
};

const TEES_CONFIG_DIR: &str = "/data/adb/teesim";
const CONFIG_FILE: &str = "config.json";
const DEFAULT_PROFILE: &str = "default";
const CONFIG_VERSION: u64 = 1;
const AUTO_INCLUDE: &str = "autoIncludeNewApps";

/// TEESimulator (`config.json`): a `profiles` map. Targets are the union of every
/// profile's `apps`; policy comes from the `default` profile's `mode`, `patchLevel`,
/// `osVersion`, device-identity fields and `autoIncludeNewApps`.
///
/// Writes follow the daemon's `ConfigStore` rules: each app keeps the profile it already
/// belongs to, new targets join `default`, every profile keeps at least one app unless it
/// auto-includes new apps, and at most one profile may auto-include.
pub(crate) struct TeeSimulatorAdapter;

const IDENTITY_FIELDS: &[(&str, &str)] = &[
    ("brand", "Brand"),
    ("device", "Device"),
    ("product", "Product"),
    ("manufacturer", "Manufacturer"),
    ("model", "Model"),
    ("serial", "Serial"),
    ("imei", "IMEI"),
    ("meid", "MEID"),
    ("imei2", "IMEI 2"),
];

impl super::ConfigAdapter for TeeSimulatorAdapter {
    fn backend(&self) -> Backend {
        Backend::TeeSimulator
    }

    fn config_dir(&self) -> &'static str {
        TEES_CONFIG_DIR
    }

    /// The `default` profile names its keybox relative to the data directory.
    fn keybox_path(&self, sysroot: &Sysroot) -> String {
        let configured = read_root(sysroot)
            .ok()
            .flatten()
            .and_then(|root| {
                root.pointer("/profiles/default/keybox")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .filter(|name| is_safe_relative(name));
        format!(
            "{TEES_CONFIG_DIR}/{}",
            configured.as_deref().unwrap_or("keybox.xml")
        )
    }

    fn policy_schema(&self) -> PolicySchema {
        let patch = |key: &str, label: &str, placeholder: &str| {
            PolicyField::new(key, label)
                .options(&["today", "system_property", "harvested", "no"])
                .placeholder(placeholder)
                .hint("today | system_property | harvested | no | YYYY-MM(-DD)")
                .validate(crate::policy::tees_patch)
        };
        let mut fields = vec![
            PolicyField::new("mode", "Operation Mode")
                .options(&["patch", "generation"])
                .placeholder("patch")
                .hint("patch | generation")
                .validate(crate::policy::tees_mode),
            patch("os_patch", "System Patch", "today"),
            patch("vendor_patch", "Vendor Patch", "YYYY-MM-05"),
            patch("boot_patch", "Boot Patch", "YYYY-MM-05"),
            PolicyField::new("os_version", "OS Version")
                .options(&["system_property", "harvested"])
                .placeholder("16 | 16.0.0 | 160000")
                .hint("system_property | harvested | version")
                .validate(crate::policy::tees_os_version),
        ];
        fields.extend(
            IDENTITY_FIELDS
                .iter()
                .map(|(key, label)| PolicyField::new(key, label)),
        );
        fields.push(PolicyField::new(AUTO_INCLUDE, "Auto-include New Apps").boolean());

        PolicySchema {
            supports_app_mode: false,
            supports_per_app_policy: false,
            default_policy: fields,
        }
    }

    fn read(&self, sysroot: &Sysroot) -> Result<ConfigData> {
        Ok(read_root(sysroot)?
            .map(|root| decode(&root))
            .unwrap_or_default())
    }

    fn write(&self, sysroot: &Sysroot, config: &ConfigData) -> Result<()> {
        let existing = read_root(sysroot)?;
        if let Some(version) = existing
            .as_ref()
            .and_then(|root| root.get("version"))
            .and_then(Value::as_u64)
            && version != CONFIG_VERSION
        {
            return Err(rule(format!(
                "config.json version {version} is not supported (expected {CONFIG_VERSION})"
            )));
        }
        let root = encode(config, existing);
        validate(sysroot, &root)?;
        let body = serde_json::to_string_pretty(&root).context("serialize TEESimulator config")?;
        write_bytes_preserving(
            &sysroot.path(format!("{TEES_CONFIG_DIR}/{CONFIG_FILE}")),
            body.as_bytes(),
        )
    }
}

fn rule(message: String) -> anyhow::Error {
    TrickyError::BackendRule(message).into()
}

fn read_root(sysroot: &Sysroot) -> Result<Option<Value>> {
    let path = sysroot.path(format!("{TEES_CONFIG_DIR}/{CONFIG_FILE}"));
    duck_core::fs::read_optional(&path)?
        .map(|raw| serde_json::from_str(&raw).context("parse TEESimulator config.json"))
        .transpose()
}

fn is_safe_relative(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('/')
        && std::path::Path::new(name)
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
}

fn decode(root: &Value) -> ConfigData {
    let profiles = root.get("profiles").and_then(Value::as_object);
    let mut targets = Vec::new();
    let mut seen = BTreeSet::new();

    for profile in profiles.into_iter().flat_map(|profiles| profiles.values()) {
        for app in apps_of(profile) {
            if seen.insert(app.to_owned()) {
                targets.push(TargetEntry {
                    package_name: app.to_owned(),
                    mode: TargetMode::Auto,
                });
            }
        }
    }

    let mut default_policy = Policy::new();
    if let Some(default) = profiles
        .and_then(|profiles| profiles.get(DEFAULT_PROFILE))
        .and_then(Value::as_object)
    {
        if let Some(patch) = default.get("patchLevel").and_then(Value::as_object) {
            copy_str(patch, "system", &mut default_policy, "os_patch");
            copy_str(patch, "vendor", &mut default_policy, "vendor_patch");
            copy_str(patch, "boot", &mut default_policy, "boot_patch");
        }
        copy_str(default, "mode", &mut default_policy, "mode");
        copy_str(default, "osVersion", &mut default_policy, "os_version");
        for (key, _) in IDENTITY_FIELDS {
            copy_str(default, key, &mut default_policy, key);
        }
        if let Some(flag) = default.get(AUTO_INCLUDE).and_then(Value::as_bool) {
            default_policy.insert(AUTO_INCLUDE.to_owned(), flag.to_string());
        }
    }

    ConfigData {
        targets,
        default_policy,
        ..ConfigData::default()
    }
}

fn apps_of(profile: &Value) -> impl Iterator<Item = &str> {
    profile
        .get("apps")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
}

fn copy_str(source: &Map<String, Value>, from: &str, policy: &mut Policy, to: &str) {
    let value = match source.get(from) {
        Some(Value::String(text)) => text.trim().to_owned(),
        Some(Value::Number(number)) => number.to_string(),
        _ => return,
    };
    if !value.is_empty() {
        policy.insert(to.to_owned(), value);
    }
}

fn encode(config: &ConfigData, existing: Option<Value>) -> Value {
    let mut root = existing
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({ "version": CONFIG_VERSION, "profiles": {} }));
    let object = root.as_object_mut().expect("root is an object");
    object.entry("version").or_insert(json!(CONFIG_VERSION));
    let profiles = object.entry("profiles").or_insert_with(|| json!({}));
    if !profiles.is_object() {
        *profiles = json!({});
    }
    let profiles = profiles.as_object_mut().expect("profiles is an object");

    // Selected apps stay in the profile that already owns them; the rest join `default`.
    let mut unassigned: Vec<&str> = config
        .targets
        .iter()
        .map(|entry| entry.package_name.as_str())
        .collect();
    for profile in profiles.values_mut().filter_map(Value::as_object_mut) {
        let kept: Vec<Value> = profile
            .get("apps")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter(|app| take(&mut unassigned, app))
            .map(|app| json!(app))
            .collect();
        profile.insert("apps".into(), Value::Array(kept));
    }

    let default = profiles
        .entry(DEFAULT_PROFILE)
        .or_insert_with(|| json!({ "keybox": "keybox.xml", "mode": "patch", "apps": [] }));
    if !default.is_object() {
        *default = json!({ "keybox": "keybox.xml", "mode": "patch", "apps": [] });
    }
    let default = default
        .as_object_mut()
        .expect("default profile is an object");
    if let Some(apps) = default.get_mut("apps").and_then(Value::as_array_mut) {
        apps.extend(unassigned.into_iter().map(|app| json!(app)));
    }

    let policy = &config.default_policy;
    let patch = default
        .entry("patchLevel")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .map(std::mem::take)
        .unwrap_or_default();
    let mut patch = patch;
    for (policy_key, config_key, fallback) in [
        ("os_patch", "system", "today"),
        ("vendor_patch", "vendor", "YYYY-MM-05"),
        ("boot_patch", "boot", "YYYY-MM-05"),
    ] {
        let value = policy.get(policy_key).map_or(fallback, String::as_str);
        patch.insert(config_key.into(), json!(value));
    }
    default.insert("patchLevel".into(), Value::Object(patch));
    default.insert(
        "mode".into(),
        json!(policy.get("mode").map_or("patch", String::as_str)),
    );
    default.insert(
        "osVersion".into(),
        json!(policy.get("os_version").map_or("", String::as_str)),
    );
    for (key, _) in IDENTITY_FIELDS {
        default.insert(
            (*key).into(),
            json!(policy.get(*key).map_or("", String::as_str)),
        );
    }
    match policy.get(AUTO_INCLUDE).map(String::as_str) {
        Some("true") => default.insert(AUTO_INCLUDE.into(), json!(true)),
        Some(_) => default.insert(AUTO_INCLUDE.into(), json!(false)),
        None => None,
    };

    root
}

/// Removes `app` from `pending`, returning whether it was there.
fn take(pending: &mut Vec<&str>, app: &str) -> bool {
    match pending.iter().position(|candidate| *candidate == app) {
        Some(index) => {
            pending.swap_remove(index);
            true
        }
        None => false,
    }
}

/// The checks TEESimulator's `ConfigStore.load` applies, so a save never leaves the daemon
/// on its last-good config with an error only visible in its log.
fn validate(sysroot: &Sysroot, root: &Value) -> Result<()> {
    let profiles = root
        .get("profiles")
        .and_then(Value::as_object)
        .filter(|profiles| !profiles.is_empty())
        .ok_or_else(|| rule("config.json has no profiles".into()))?;

    let mut auto_include: Option<&str> = None;
    for (id, profile) in profiles {
        let keybox = profile
            .get("keybox")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or_default();
        if !is_safe_relative(keybox) {
            return Err(rule(format!("profile '{id}' has no valid keybox")));
        }
        if !sysroot
            .path(format!("{TEES_CONFIG_DIR}/{keybox}"))
            .is_file()
        {
            return Err(rule(format!(
                "profile '{id}' keybox {keybox} is missing; install a keybox first"
            )));
        }
        let mode = profile
            .get("mode")
            .and_then(Value::as_str)
            .unwrap_or("patch");
        if !crate::policy::tees_mode(mode) {
            return Err(rule(format!("profile '{id}' has invalid mode '{mode}'")));
        }
        let auto = profile
            .get(AUTO_INCLUDE)
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if auto {
            if let Some(other) = auto_include {
                return Err(rule(format!(
                    "profiles '{other}' and '{id}' both auto-include new apps; only one may"
                )));
            }
            auto_include = Some(id);
        }
        if apps_of(profile).next().is_none() && !auto {
            return Err(rule(format!(
                "profile '{id}' would have no apps; keep one of its apps selected or enable auto-include"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use duck_core::Sysroot;
    use serde_json::json;

    use super::{decode, encode, validate};
    use crate::model::{TargetEntry, TargetMode};

    fn targets(names: &[&str]) -> Vec<TargetEntry> {
        names
            .iter()
            .map(|name| TargetEntry {
                package_name: (*name).into(),
                mode: TargetMode::Auto,
            })
            .collect()
    }

    #[test]
    fn decodes_apps_mode_and_patch_level() {
        let root = json!({
            "version": 1,
            "profiles": {
                "default": {
                    "mode": "generation",
                    "patchLevel": { "system": "today", "vendor": "2026-01-05", "boot": "no" },
                    "osVersion": 160000,
                    "autoIncludeNewApps": true,
                    "apps": ["com.a", "com.b"],
                },
                "bank": { "apps": ["com.b", "com.c"] }
            }
        });

        let config = decode(&root);
        assert_eq!(config.targets.len(), 3);
        assert_eq!(config.default_policy["mode"], "generation");
        assert_eq!(config.default_policy["vendor_patch"], "2026-01-05");
        assert_eq!(config.default_policy["os_version"], "160000");
        assert_eq!(config.default_policy["autoIncludeNewApps"], "true");
    }

    #[test]
    fn apps_keep_their_profile_and_new_ones_join_default() {
        let existing = json!({
            "version": 1,
            "custom": "kept",
            "profiles": {
                "default": { "keybox": "keybox.xml", "apps": ["com.old"] },
                "bank": { "keybox": "bank.xml", "mode": "generation", "apps": ["com.bank", "com.gone"] }
            }
        });
        let mut config = decode(&existing);
        config.targets = targets(&["com.bank", "com.new", "com.old"]);

        let encoded = encode(&config, Some(existing));
        assert_eq!(encoded["custom"], "kept");
        assert_eq!(encoded["profiles"]["bank"]["apps"], json!(["com.bank"]));
        assert_eq!(encoded["profiles"]["bank"]["mode"], "generation");
        assert_eq!(
            encoded["profiles"]["default"]["apps"],
            json!(["com.old", "com.new"])
        );
        assert_eq!(
            encoded["profiles"]["default"]["patchLevel"]["system"],
            "today"
        );
    }

    #[test]
    fn validation_matches_the_daemon() {
        let root = tempfile::tempdir().unwrap();
        let sysroot = Sysroot::new(root.path());
        let dir = sysroot.path("/data/adb/teesim");
        fs::create_dir_all(&dir).unwrap();
        let config = |profiles: serde_json::Value| json!({ "version": 1, "profiles": profiles });

        let missing_keybox =
            config(json!({ "default": { "keybox": "keybox.xml", "apps": ["com.a"] } }));
        assert!(validate(&sysroot, &missing_keybox).is_err());

        fs::write(dir.join("keybox.xml"), "<AndroidAttestation/>").unwrap();
        validate(&sysroot, &missing_keybox).unwrap();

        let empty = config(json!({
            "default": { "keybox": "keybox.xml", "apps": ["com.a"] },
            "bank": { "keybox": "keybox.xml", "apps": [] }
        }));
        let error = validate(&sysroot, &empty).unwrap_err().to_string();
        assert!(error.contains("'bank'"), "{error}");

        let two_auto = config(json!({
            "default": { "keybox": "keybox.xml", "apps": [], "autoIncludeNewApps": true },
            "bank": { "keybox": "keybox.xml", "apps": [], "autoIncludeNewApps": true }
        }));
        assert!(validate(&sysroot, &two_auto).is_err());

        let escape = config(json!({ "default": { "keybox": "../x.xml", "apps": ["com.a"] } }));
        assert!(validate(&sysroot, &escape).is_err());
    }
}
