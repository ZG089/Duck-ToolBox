use anyhow::Result;
use duck_core::{Sysroot, fs::write_bytes_preserving};

use crate::model::{
    Backend, ConfigData, Policy, PolicyField, PolicySchema, TargetEntry, TargetMode,
};

pub(crate) const TS_CONFIG_DIR: &str = "/data/adb/tricky_store";

/// Tricky Store (config.ini): `[target]` list with `!`/`?` mode markers, a
/// `[default_policy]` section and optional per-package `[pkg]` sections.
pub(crate) struct TrickyStoreAdapter;

impl super::ConfigAdapter for TrickyStoreAdapter {
    fn backend(&self) -> Backend {
        Backend::TrickyStore
    }

    fn config_dir(&self) -> &'static str {
        TS_CONFIG_DIR
    }

    fn policy_schema(&self) -> PolicySchema {
        PolicySchema {
            supports_app_mode: true,
            supports_per_app_policy: true,
            default_policy: patch_fields(),
        }
    }

    fn read(&self, sysroot: &Sysroot) -> Result<ConfigData> {
        let path = sysroot.path(format!("{TS_CONFIG_DIR}/config.ini"));
        let Some(raw) = duck_core::fs::read_optional(&path)? else {
            return Ok(ConfigData::default());
        };
        Ok(parse_ini(&raw))
    }

    fn write(&self, sysroot: &Sysroot, config: &ConfigData) -> Result<()> {
        let path = sysroot.path(format!("{TS_CONFIG_DIR}/config.ini"));
        write_bytes_preserving(&path, serialize_ini(config).as_bytes())
    }
}

pub(crate) fn patch_fields() -> Vec<PolicyField> {
    vec![
        PolicyField::new("os_patch", "System Patch")
            .options(&["prop", "no"])
            .placeholder("YYYYMM")
            .max_length(6)
            .hint("YYYYMM | prop | no")
            .validate(crate::policy::ts_month),
        PolicyField::new("vendor_patch", "Vendor Patch")
            .options(&["prop", "no"])
            .placeholder("YYYYMMDD")
            .max_length(8)
            .hint("YYYYMMDD | prop | no")
            .validate(crate::policy::ts_day),
        PolicyField::new("boot_patch", "Boot Patch")
            .options(&["prop", "no"])
            .placeholder("YYYYMMDD")
            .max_length(8)
            .hint("YYYYMMDD | prop | no")
            .validate(crate::policy::ts_day),
    ]
}

fn parse_ini(raw: &str) -> ConfigData {
    let mut config = ConfigData::default();
    let mut section: Option<String> = None;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(name) = trimmed
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        {
            section = Some(name.trim().to_owned());
            continue;
        }

        match section.as_deref() {
            Some("target") => {
                let (package_name, mode) = TargetMode::split(trimmed);
                if !package_name.is_empty() {
                    config.targets.push(TargetEntry { package_name, mode });
                }
            }
            Some("default_policy") => insert_kv(&mut config.default_policy, trimmed),
            Some(pkg) => {
                let entry = config.per_app_policy.entry(pkg.to_owned()).or_default();
                insert_kv(entry, trimmed);
            }
            None => {}
        }
    }

    config
}

fn insert_kv(policy: &mut Policy, line: &str) {
    if let Some((key, value)) = line.split_once('=') {
        policy.insert(key.trim().to_owned(), value.trim().to_owned());
    }
}

fn serialize_ini(config: &ConfigData) -> String {
    let mut sections = Vec::new();

    if !config.default_policy.is_empty() {
        sections.push(serialize_section("default_policy", &config.default_policy));
    }

    let mut target_lines = vec!["[target]".to_owned()];
    for entry in &config.targets {
        target_lines.push(format!("{}{}", entry.package_name, entry.mode.marker()));
    }
    sections.push(target_lines.join("\n"));

    for (pkg, policy) in &config.per_app_policy {
        if !policy.is_empty() {
            sections.push(serialize_section(pkg, policy));
        }
    }

    let mut body = sections.join("\n\n");
    body.push('\n');
    body
}

fn serialize_section(name: &str, policy: &Policy) -> String {
    let mut lines = vec![format!("[{name}]")];
    for (key, value) in policy {
        lines.push(format!("{key} = {value}"));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{parse_ini, serialize_ini};
    use crate::model::TargetMode;

    #[test]
    fn round_trips_targets_modes_and_policies() {
        let raw = "[default_policy]\nos_patch = prop\n\n[target]\ncom.a\ncom.b!\ncom.c?\n\n[com.b]\nboot_patch = 20260101\n";
        let config = parse_ini(raw);

        assert_eq!(config.targets.len(), 3);
        assert_eq!(config.targets[1].mode, TargetMode::Generate);
        assert_eq!(config.targets[2].mode, TargetMode::Hack);
        assert_eq!(config.default_policy["os_patch"], "prop");
        assert_eq!(config.per_app_policy["com.b"]["boot_patch"], "20260101");

        let reparsed = parse_ini(&serialize_ini(&config));
        assert_eq!(reparsed.targets, config.targets);
        assert_eq!(reparsed.default_policy, config.default_policy);
        assert_eq!(reparsed.per_app_policy, config.per_app_policy);
    }
}
