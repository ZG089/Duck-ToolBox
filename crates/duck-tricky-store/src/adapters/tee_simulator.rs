use anyhow::{Context, Result};
use duck_core::{Sysroot, fs::write_bytes_preserving};
use serde_json::{Map, Value, json};

use crate::{
    error::TrickyError,
    model::{Backend, ConfigData, Policy, PolicyField, PolicySchema, TargetEntry, TargetMode},
};

const TEES_CONFIG_DIR: &str = "/data/adb/teesim";

/// TEESimulator (`config.json`): a `profiles` map. Targets are the union of every
/// profile's `apps`; policy comes from the `default` profile's `patchLevel`, `osVersion`
/// and device-identity fields.
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
        let configured =
            duck_core::fs::read_optional(&sysroot.path(format!("{TEES_CONFIG_DIR}/config.json")))
                .ok()
                .flatten()
                .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
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
        let mut fields = vec![
            PolicyField::new("os_patch", "System Patch")
                .options(&["today", "harvested", "system_property", "no"])
                .placeholder("YYYY-MM-DD")
                .hint("today | harvested | system_property | no | date")
                .validate(crate::policy::tees_patch),
            PolicyField::new("vendor_patch", "Vendor Patch")
                .options(&["today", "harvested", "system_property", "no"])
                .placeholder("YYYY-MM-05")
                .hint("today | harvested | system_property | no | date")
                .validate(crate::policy::tees_patch),
            PolicyField::new("boot_patch", "Boot Patch")
                .options(&["today", "harvested", "system_property", "no"])
                .placeholder("YYYY-MM-05")
                .hint("today | harvested | system_property | no | date")
                .validate(crate::policy::tees_patch),
            PolicyField::new("os_version", "OS Version")
                .options(&["harvested", "system_property", "no"])
                .placeholder("16 or 16.0.0")
                .hint("harvested | system_property | no | version")
                .validate(crate::policy::tees_os_version),
        ];
        fields.extend(
            IDENTITY_FIELDS
                .iter()
                .map(|(key, label)| PolicyField::new(key, label)),
        );

        PolicySchema {
            supports_app_mode: false,
            supports_per_app_policy: false,
            default_policy: fields,
        }
    }

    fn read(&self, sysroot: &Sysroot) -> Result<ConfigData> {
        let path = sysroot.path(format!("{TEES_CONFIG_DIR}/config.json"));
        let Some(raw) = duck_core::fs::read_optional(&path)? else {
            return Ok(ConfigData::default());
        };
        let root: Value = serde_json::from_str(&raw).context("parse TEESimulator config.json")?;
        Ok(decode(&root))
    }

    fn write(&self, sysroot: &Sysroot, config: &ConfigData) -> Result<()> {
        let path = sysroot.path(format!("{TEES_CONFIG_DIR}/config.json"));
        let existing = duck_core::fs::read_optional(&path)?
            .and_then(|raw| serde_json::from_str::<Value>(&raw).ok());
        if config.targets.is_empty() && !auto_includes_new_apps(existing.as_ref()) {
            return Err(TrickyError::BackendRule(
                "TEESimulator rejects a profile without apps; select at least one target".into(),
            )
            .into());
        }
        let root = encode(config, existing);
        let body = serde_json::to_string_pretty(&root).context("serialize TEESimulator config")?;
        write_bytes_preserving(&path, body.as_bytes())
    }
}

fn auto_includes_new_apps(root: Option<&Value>) -> bool {
    root.and_then(|root| root.get("profiles"))
        .and_then(Value::as_object)
        .is_some_and(|profiles| {
            profiles.values().any(|profile| {
                profile
                    .get("autoIncludeNewApps")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            })
        })
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
    let mut seen = std::collections::BTreeSet::new();

    if let Some(profiles) = profiles {
        for profile in profiles.values() {
            for app in profile
                .get("apps")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if let Some(app) = app.as_str()
                    && seen.insert(app.to_owned())
                {
                    targets.push(TargetEntry {
                        package_name: app.to_owned(),
                        mode: TargetMode::Auto,
                    });
                }
            }
        }
    }

    let default = profiles.and_then(|profiles| profiles.get("default"));
    let mut default_policy = Policy::new();
    if let Some(default) = default {
        if let Some(patch) = default.get("patchLevel").and_then(Value::as_object) {
            copy_str(patch, "system", &mut default_policy, "os_patch");
            copy_str(patch, "vendor", &mut default_policy, "vendor_patch");
            copy_str(patch, "boot", &mut default_policy, "boot_patch");
        }
        copy_str(
            default.as_object().unwrap_or(&Map::new()),
            "osVersion",
            &mut default_policy,
            "os_version",
        );
        if let Some(profile) = default.as_object() {
            for (key, _) in IDENTITY_FIELDS {
                copy_str(profile, key, &mut default_policy, key);
            }
        }
    }

    ConfigData {
        targets,
        default_policy,
        ..ConfigData::default()
    }
}

fn copy_str(source: &Map<String, Value>, from: &str, policy: &mut Policy, to: &str) {
    if let Some(value) = source.get(from).and_then(value_to_string)
        && !value.is_empty()
    {
        policy.insert(to.to_owned(), value);
    }
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

fn encode(config: &ConfigData, existing: Option<Value>) -> Value {
    let mut root = existing.unwrap_or_else(|| json!({ "version": 1, "profiles": {} }));
    if !root.is_object() {
        root = json!({ "version": 1, "profiles": {} });
    }
    let object = root.as_object_mut().expect("root is an object");
    object.entry("version").or_insert(json!(1));
    let profiles = object
        .entry("profiles")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .expect("profiles is an object");

    let default = profiles
        .entry("default")
        .or_insert_with(|| json!({ "keybox": "keybox.xml" }));
    let default = default
        .as_object_mut()
        .expect("default profile is an object");

    let policy = &config.default_policy;
    let patch = json!({
        "system": policy.get("os_patch").cloned().unwrap_or_else(|| "today".into()),
        "vendor": policy.get("vendor_patch").cloned().unwrap_or_else(|| "YYYY-MM-05".into()),
        "boot": policy.get("boot_patch").cloned().unwrap_or_else(|| "YYYY-MM-05".into()),
    });
    default.insert("patchLevel".into(), patch);
    set_or_clear(default, "osVersion", policy.get("os_version"));
    for (key, _) in IDENTITY_FIELDS {
        set_or_clear(default, key, policy.get(*key));
    }

    // Reassign every target to the default profile and clear apps from other profiles.
    let apps: Vec<Value> = config
        .targets
        .iter()
        .map(|entry| json!(entry.package_name))
        .collect();
    default.insert("apps".into(), Value::Array(apps));
    for (name, profile) in profiles.iter_mut() {
        if name != "default"
            && let Some(profile) = profile.as_object_mut()
        {
            profile.insert("apps".into(), json!([]));
        }
    }

    root
}

fn set_or_clear(object: &mut Map<String, Value>, key: &str, value: Option<&String>) {
    match value {
        Some(value) if !value.is_empty() => {
            object.insert(key.to_owned(), json!(value));
        }
        _ => {
            object.insert(key.to_owned(), json!(""));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{decode, encode};
    use serde_json::json;

    #[test]
    fn decodes_apps_and_patch_level() {
        let root = json!({
            "version": 1,
            "profiles": {
                "default": {
                    "patchLevel": { "system": "today", "vendor": "2026-01-05", "boot": "no" },
                    "osVersion": "16",
                    "apps": ["com.a", "com.b"],
                }
            }
        });

        let config = decode(&root);
        assert_eq!(config.targets.len(), 2);
        assert_eq!(config.default_policy["os_patch"], "today");
        assert_eq!(config.default_policy["vendor_patch"], "2026-01-05");
        assert_eq!(config.default_policy["os_version"], "16");
    }

    #[test]
    fn encode_reassigns_targets_to_default_profile() {
        let config = decode(&json!({
            "profiles": { "default": { "apps": [] }, "extra": { "apps": ["com.old"] } }
        }));
        let mut config = config;
        config.targets = vec![crate::model::TargetEntry {
            package_name: "com.new".into(),
            mode: crate::model::TargetMode::Auto,
        }];

        let encoded = encode(&config, None);
        let default_apps = encoded["profiles"]["default"]["apps"].as_array().unwrap();
        assert_eq!(default_apps.len(), 1);
        assert_eq!(default_apps[0], "com.new");
    }
}
