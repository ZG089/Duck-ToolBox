use anyhow::Result;
use duck_core::{Sysroot, fs::write_bytes_preserving};

use crate::model::{Backend, ConfigData, Policy, PolicySchema, TargetEntry, TargetMode};

use super::tricky_store::TS_CONFIG_DIR;

/// Classic Tricky Store: a plain `target.txt` (with `!`/`?` markers) plus an optional
/// `security_patch.txt`. There is no per-app policy.
pub(crate) struct TrickyStoreLegacyAdapter;

impl super::ConfigAdapter for TrickyStoreLegacyAdapter {
    fn backend(&self) -> Backend {
        Backend::TrickyStoreLegacy
    }

    fn config_dir(&self) -> &'static str {
        TS_CONFIG_DIR
    }

    fn policy_schema(&self) -> PolicySchema {
        PolicySchema {
            supports_app_mode: true,
            supports_per_app_policy: false,
            default_policy: super::tricky_store::patch_fields(),
        }
    }

    fn read(&self, sysroot: &Sysroot) -> Result<ConfigData> {
        let targets =
            duck_core::fs::read_optional(&sysroot.path(format!("{TS_CONFIG_DIR}/target.txt")))?
                .map(|raw| parse_targets(&raw))
                .unwrap_or_default();
        let default_policy = duck_core::fs::read_optional(
            &sysroot.path(format!("{TS_CONFIG_DIR}/security_patch.txt")),
        )?
        .map(|raw| parse_security_patch(&raw))
        .unwrap_or_default();

        Ok(ConfigData {
            targets,
            default_policy,
            ..ConfigData::default()
        })
    }

    fn write(&self, sysroot: &Sysroot, config: &ConfigData) -> Result<()> {
        let target_path = sysroot.path(format!("{TS_CONFIG_DIR}/target.txt"));
        let body: String = config
            .targets
            .iter()
            .map(|entry| format!("{}{}\n", entry.package_name, entry.mode.marker()))
            .collect();
        write_bytes_preserving(&target_path, body.as_bytes())?;

        let sp_path = sysroot.path(format!("{TS_CONFIG_DIR}/security_patch.txt"));
        let sp = serialize_security_patch(&config.default_policy);
        if sp.is_empty() {
            let _ = std::fs::remove_file(&sp_path);
        } else {
            write_bytes_preserving(&sp_path, sp.as_bytes())?;
        }
        Ok(())
    }
}

fn parse_targets(raw: &str) -> Vec<TargetEntry> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (package_name, mode) = TargetMode::split(line);
            TargetEntry { package_name, mode }
        })
        .filter(|entry| !entry.package_name.is_empty())
        .collect()
}

/// `security_patch.txt` keys map onto the shared `*_patch` policy keys. A bare value (no
/// `=`) sets all three, with the system level trimmed to `YYYYMM`.
fn parse_security_patch(raw: &str) -> Policy {
    let mut policy = Policy::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        match trimmed.split_once('=') {
            Some(("system", value)) => insert(&mut policy, "os_patch", value),
            Some(("boot", value)) => insert(&mut policy, "boot_patch", value),
            Some(("vendor", value)) => insert(&mut policy, "vendor_patch", value),
            Some(("all", value)) => set_all(&mut policy, value),
            None => set_all(&mut policy, trimmed),
            Some(_) => {}
        }
    }
    policy
}

/// A bare value or `all=` sets all three levels, with the system level trimmed to `YYYYMM`.
fn set_all(policy: &mut Policy, value: &str) {
    insert(policy, "os_patch", strip_day(value));
    insert(policy, "boot_patch", value);
    insert(policy, "vendor_patch", value);
}

fn insert(policy: &mut Policy, key: &str, value: &str) {
    policy.insert(key.to_owned(), value.trim().to_owned());
}

fn strip_day(value: &str) -> &str {
    let value = value.trim();
    if value.len() == 8 && value.chars().all(|c| c.is_ascii_digit()) {
        &value[..6]
    } else {
        value
    }
}

fn serialize_security_patch(policy: &Policy) -> String {
    let is_noop = |key: &str| policy.get(key).is_none_or(|value| value == "no");
    if ["os_patch", "boot_patch", "vendor_patch"]
        .iter()
        .all(|key| is_noop(key))
    {
        return String::new();
    }

    let mut lines = Vec::new();
    if let Some(value) = policy.get("os_patch") {
        lines.push(format!("system={value}"));
    }
    if let Some(value) = policy.get("boot_patch") {
        lines.push(format!("boot={value}"));
    }
    if let Some(value) = policy.get("vendor_patch") {
        lines.push(format!("vendor={value}"));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{parse_security_patch, parse_targets, serialize_security_patch};
    use crate::model::TargetMode;

    #[test]
    fn parses_targets_with_modes() {
        let targets = parse_targets("# comment\ncom.a\ncom.b!\ncom.c?\n\n");
        assert_eq!(targets.len(), 3);
        assert_eq!(targets[1].mode, TargetMode::Generate);
    }

    #[test]
    fn bare_security_patch_sets_all_levels() {
        let policy = parse_security_patch("20241101\n");
        assert_eq!(policy["os_patch"], "202411");
        assert_eq!(policy["boot_patch"], "20241101");
        assert_eq!(policy["vendor_patch"], "20241101");
    }

    #[test]
    fn noop_policy_serializes_empty() {
        let mut policy = std::collections::BTreeMap::new();
        policy.insert("os_patch".to_owned(), "no".to_owned());
        assert!(serialize_security_patch(&policy).is_empty());
    }
}
