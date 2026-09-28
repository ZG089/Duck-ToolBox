use anyhow::{Context, Result};
use duck_core::{Sysroot, fs::write_bytes_preserving};
use toml_edit::{Array, DocumentMut, Item, Table, Value};

mod injector;

use crate::{
    error::TrickyError,
    model::{Backend, ConfigData, Policy, PolicyField, PolicySchema, TargetEntry, TargetMode},
};

const OMK_CONFIG_DIR: &str = "/data/misc/keystore/omk";
const INJECTOR_FILE: &str = "injector.toml";
const CONFIG_FILE: &str = "config.toml";
/// Every `[trust]` key of OhMyKeymint's `RawTrustConfig` (`src/config.rs`).
const TRUST_FIELDS: &[&str] = &[
    "os_version",
    "security_patch",
    "os_patchlevel",
    "vendor_patchlevel",
    "boot_patchlevel",
    "vb_key",
    "vb_hash",
    "verified_boot_state",
    "device_locked",
];
const BOOLEAN_FIELDS: &[&str] = &["verified_boot_state", "device_locked"];
/// Trust keys OhMyKeymint applies only after a keymint restart (`trust_changed_beyond_patchlevels`
/// and `boot_patchlevel_changed` in `src/config.rs`); patch levels reload on the fly.
const RESTART_FIELDS: &[&str] = &[
    "os_version",
    "vb_key",
    "vb_hash",
    "verified_boot_state",
    "device_locked",
    "boot_patchlevel",
];

/// OhMyKeymint: the `scoop` target list lives in `injector.toml`; the `[trust]` policy
/// lives in `config.toml`. Neither file has a per-app mode. Both are edited in place with
/// `toml_edit`, so comments and unrelated settings survive a save.
pub(crate) struct OhMyKeymintAdapter;

impl super::ConfigAdapter for OhMyKeymintAdapter {
    fn backend(&self) -> Backend {
        Backend::OhMyKeymint
    }

    fn config_dir(&self) -> &'static str {
        OMK_CONFIG_DIR
    }

    fn policy_schema(&self) -> PolicySchema {
        let patch = |key: &str, label: &str| {
            PolicyField::new(key, label)
                .options(&["auto", "latest"])
                .placeholder("YYYY-MM-DD")
                .max_length(10)
                .hint("auto | latest | YYYY-MM-DD")
                .validate(crate::policy::omk_security_patch)
        };
        let boot_hash = |key: &str, label: &str| {
            PolicyField::new(key, label)
                .options(&["auto", "random"])
                .placeholder("64 hex chars")
                .max_length(64)
                .multiline()
                .hint("auto | random | 64 hex chars")
                .validate(crate::policy::omk_vb)
        };
        PolicySchema {
            supports_app_mode: false,
            supports_per_app_policy: false,
            default_policy: vec![
                PolicyField::new("os_version", "OS Version")
                    .options(&["auto"])
                    .placeholder("16")
                    .max_length(2)
                    .hint("auto | Android major version")
                    .validate(crate::policy::omk_os_version),
                patch("security_patch", "Security Patch"),
                patch("os_patchlevel", "OS Patch Level"),
                patch("vendor_patchlevel", "Vendor Patch Level"),
                PolicyField::new("boot_patchlevel", "Boot Patch Level")
                    .options(&["auto", "latest"])
                    .placeholder("YYYY-MM-DD")
                    .max_length(10)
                    .hint("auto | latest | YYYY-MM-DD | number")
                    .validate(crate::policy::omk_boot_patchlevel),
                boot_hash("vb_key", "VB Key"),
                boot_hash("vb_hash", "VB Hash"),
                PolicyField::new("verified_boot_state", "Verified Boot State").boolean(),
                PolicyField::new("device_locked", "Device Locked").boolean(),
            ],
        }
    }

    fn restart_keys(&self) -> &'static [&'static str] {
        RESTART_FIELDS
    }

    fn read(&self, sysroot: &Sysroot) -> Result<ConfigData> {
        let targets = read_injector(sysroot)?
            .and_then(|document| document.get("scoop").and_then(Item::as_array).cloned())
            .map(|scoop| {
                scoop
                    .iter()
                    .filter_map(Value::as_str)
                    .filter(|name| !name.is_empty())
                    .map(|name| TargetEntry {
                        package_name: name.to_owned(),
                        mode: TargetMode::Auto,
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut default_policy = Policy::new();
        if let Some(document) = read_document(sysroot, CONFIG_FILE)?
            && let Some(trust) = document.get("trust").and_then(Item::as_table_like)
        {
            for key in TRUST_FIELDS {
                if let Some(value) = trust.get(key).and_then(Item::as_value) {
                    default_policy.insert((*key).to_owned(), scalar_to_string(value));
                }
            }
        }

        Ok(ConfigData {
            targets,
            default_policy,
            ..ConfigData::default()
        })
    }

    fn write(&self, sysroot: &Sysroot, config: &ConfigData) -> Result<()> {
        let mut injector = read_injector(sysroot)?.unwrap_or_default();
        let names: Vec<&str> = config
            .targets
            .iter()
            .map(|entry| entry.package_name.as_str())
            .collect();
        let scoop = Item::Value(Value::Array(multiline_array(&names)));
        // Replace only the value: the comment above `scoop` belongs to its key.
        match injector.get_mut("scoop") {
            Some(item) => *item = scoop,
            None => {
                injector.insert("scoop", scoop);
            }
        }
        let path = sysroot.path(format!("{OMK_CONFIG_DIR}/{INJECTOR_FILE}"));
        write_bytes_preserving(&path, injector::render(&injector).as_bytes())?;

        if config.default_policy.is_empty() {
            return Ok(());
        }
        // OhMyKeymint seeds a complete config.toml on first start and refuses one that lacks
        // required trust keys, so a partial file must never be created from scratch.
        let Some(mut main) = read_document(sysroot, CONFIG_FILE)? else {
            return Err(TrickyError::BackendRule(
                "OhMyKeymint has not created config.toml yet; reboot once with it enabled".into(),
            )
            .into());
        };
        let trust = main
            .entry("trust")
            .or_insert_with(|| Item::Table(Table::new()))
            .as_table_like_mut()
            .context("OhMyKeymint [trust] is not a table")?;
        for key in TRUST_FIELDS {
            if let Some(text) = config.default_policy.get(*key) {
                let mut value = trust_value(key, text);
                if let Some(existing) = trust.get(key).and_then(Item::as_value) {
                    *value.decor_mut() = existing.decor().clone();
                }
                trust.insert(key, Item::Value(value));
            }
        }
        write_document(sysroot, CONFIG_FILE, &main)
    }
}

fn read_injector(sysroot: &Sysroot) -> Result<Option<DocumentMut>> {
    let path = sysroot.path(format!("{OMK_CONFIG_DIR}/{INJECTOR_FILE}"));
    duck_core::fs::read_optional(&path)?
        .map(|raw| injector::parse(&raw))
        .transpose()
}

fn read_document(sysroot: &Sysroot, file: &str) -> Result<Option<DocumentMut>> {
    let path = sysroot.path(format!("{OMK_CONFIG_DIR}/{file}"));
    match duck_core::fs::read_optional(&path)? {
        Some(raw) => Ok(Some(
            raw.parse::<DocumentMut>()
                .with_context(|| format!("parse OhMyKeymint {file}"))?,
        )),
        None => Ok(None),
    }
}

fn write_document(sysroot: &Sysroot, file: &str, document: &DocumentMut) -> Result<()> {
    let path = sysroot.path(format!("{OMK_CONFIG_DIR}/{file}"));
    write_bytes_preserving(&path, document.to_string().as_bytes())
}

/// One package per line, like OhMyKeymint's own template.
fn multiline_array(items: &[&str]) -> Array {
    let mut array: Array = items.iter().copied().collect();
    if !array.is_empty() {
        for value in array.iter_mut() {
            value.decor_mut().set_prefix("\n  ");
        }
        array.set_trailing("\n");
        array.set_trailing_comma(true);
    }
    array
}

fn scalar_to_string(value: &Value) -> String {
    match value {
        Value::String(text) => text.value().clone(),
        Value::Integer(number) => number.value().to_string(),
        Value::Boolean(flag) => flag.value().to_string(),
        other => other.to_string().trim().to_owned(),
    }
}

/// `os_version` is an integer when numeric and the two state flags are booleans; every
/// other trust field is a string, matching OhMyKeymint's serde types.
fn trust_value(key: &str, text: &str) -> Value {
    if key == "os_version"
        && let Ok(number) = text.parse::<i64>()
    {
        return Value::from(number);
    }
    if BOOLEAN_FIELDS.contains(&key)
        && let Ok(flag) = text.parse::<bool>()
    {
        return Value::from(flag);
    }
    Value::from(text)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use duck_core::Sysroot;

    use super::{OhMyKeymintAdapter, scalar_to_string, trust_value};
    use crate::{
        adapters::ConfigAdapter,
        model::{ConfigData, TargetEntry},
    };

    #[test]
    fn typed_trust_values() {
        assert_eq!(trust_value("os_version", "15").as_integer(), Some(15));
        assert_eq!(trust_value("os_version", "auto").as_str(), Some("auto"));
        assert_eq!(trust_value("device_locked", "false").as_bool(), Some(false));
        assert_eq!(trust_value("vb_key", "auto").as_str(), Some("auto"));
        assert_eq!(scalar_to_string(&trust_value("os_version", "15")), "15");
    }

    #[test]
    fn edits_keep_comments_and_unknown_sections() {
        let root = tempfile::tempdir().unwrap();
        let sysroot = Sysroot::new(root.path());
        let dir = sysroot.path("/data/misc/keystore/omk");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("injector.toml"),
            "# keep me\nscoop = [\"old.app\"]\n\n[filter]\nenabled = true\n",
        )
        .unwrap();
        fs::write(
            dir.join("config.toml"),
            "version = 2\n[trust]\nos_version = \"auto\" # comment\nsecurity_patch = \"auto\"\ndevice_locked = true\n[crypto]\nroot_kek_seed = [1]\n",
        )
        .unwrap();

        let adapter = OhMyKeymintAdapter;
        let mut config = adapter.read(&sysroot).unwrap();
        assert_eq!(config.targets[0].package_name, "old.app");
        assert_eq!(config.default_policy["device_locked"], "true");

        config.targets = vec![TargetEntry {
            package_name: "com.example.bank".into(),
            mode: Default::default(),
        }];
        config
            .default_policy
            .insert("os_version".into(), "16".into());
        config
            .default_policy
            .insert("device_locked".into(), "false".into());
        adapter.write(&sysroot, &config).unwrap();

        let injector = fs::read_to_string(dir.join("injector.toml")).unwrap();
        assert!(injector.contains("# keep me"));
        assert!(injector.contains("[filter]"));
        assert!(injector.contains("\n  \"com.example.bank\",\n]"));
        assert!(
            !injector.contains("version"),
            "released OhMyKeymint rejects unknown keys"
        );
        let main = fs::read_to_string(dir.join("config.toml")).unwrap();
        assert!(main.contains("os_version = 16 # comment"));
        assert!(main.contains("device_locked = false"));
        assert!(main.contains("[crypto]"));
    }

    #[test]
    fn never_creates_a_partial_config() {
        let root = tempfile::tempdir().unwrap();
        let sysroot = Sysroot::new(root.path());
        let mut config = ConfigData::default();
        config
            .default_policy
            .insert("os_version".into(), "16".into());

        assert!(OhMyKeymintAdapter.write(&sysroot, &config).is_err());
        assert!(!sysroot.path("/data/misc/keystore/omk/config.toml").exists());
        assert!(
            sysroot
                .path("/data/misc/keystore/omk/injector.toml")
                .exists()
        );
    }
}
