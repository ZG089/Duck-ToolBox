use anyhow::{Context, Result};
use duck_core::{Sysroot, fs::write_bytes_preserving};
use toml::{Table, Value};

use crate::model::{
    Backend, ConfigData, Policy, PolicyField, PolicySchema, TargetEntry, TargetMode,
};

const OMK_CONFIG_DIR: &str = "/data/misc/keystore/omk";
const TRUST_FIELDS: &[&str] = &["os_version", "security_patch", "vb_key", "vb_hash"];

/// OhMyKeymint: the `scoop` target list lives in `injector.toml`; the `[trust]` policy
/// lives in `config.toml`. Neither file has a per-app mode.
pub(crate) struct OhMyKeymintAdapter;

impl super::ConfigAdapter for OhMyKeymintAdapter {
    fn backend(&self) -> Backend {
        Backend::OhMyKeymint
    }

    fn config_dir(&self) -> &'static str {
        OMK_CONFIG_DIR
    }

    fn policy_schema(&self) -> PolicySchema {
        PolicySchema {
            supports_app_mode: false,
            supports_per_app_policy: false,
            default_policy: vec![
                PolicyField::new("os_version", "OS Version")
                    .options(&["auto"])
                    .placeholder("15")
                    .hint("auto | number")
                    .validate(crate::policy::omk_os_version),
                PolicyField::new("security_patch", "Security Patch")
                    .options(&["auto", "latest"])
                    .placeholder("YYYY-MM-DD")
                    .max_length(10)
                    .hint("auto | latest | YYYY-MM-DD")
                    .validate(crate::policy::omk_security_patch),
                PolicyField::new("vb_key", "VB Key")
                    .options(&["auto", "random"])
                    .placeholder("64 hex chars")
                    .max_length(64)
                    .multiline()
                    .hint("auto | random | 64 hex chars")
                    .validate(crate::policy::omk_vb),
                PolicyField::new("vb_hash", "VB Hash")
                    .options(&["auto", "random"])
                    .placeholder("64 hex chars")
                    .max_length(64)
                    .multiline()
                    .hint("auto | random | 64 hex chars")
                    .validate(crate::policy::omk_vb),
            ],
        }
    }

    fn read(&self, sysroot: &Sysroot) -> Result<ConfigData> {
        let targets = read_table(sysroot, "injector.toml")?
            .and_then(|table| table.get("scoop").and_then(Value::as_array).cloned())
            .map(|scoop| {
                scoop
                    .into_iter()
                    .filter_map(|value| value.as_str().map(str::to_owned))
                    .filter(|name| !name.is_empty())
                    .map(|package_name| TargetEntry {
                        package_name,
                        mode: TargetMode::Auto,
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut default_policy = Policy::new();
        if let Some(trust) = read_table(sysroot, "config.toml")?
            .and_then(|table| table.get("trust").and_then(Value::as_table).cloned())
        {
            for key in TRUST_FIELDS {
                if let Some(value) = trust.get(*key) {
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
        let mut injector = read_table(sysroot, "injector.toml")?.unwrap_or_default();
        injector.entry("version").or_insert(Value::Integer(1));
        let scoop = config
            .targets
            .iter()
            .map(|entry| Value::String(entry.package_name.clone()))
            .collect();
        injector.insert("scoop".into(), Value::Array(scoop));
        write_table(sysroot, "injector.toml", &injector)?;

        let mut main = read_table(sysroot, "config.toml")?.unwrap_or_default();
        main.entry("version").or_insert(Value::Integer(2));
        let trust = main
            .entry("trust")
            .or_insert_with(|| Value::Table(Table::new()))
            .as_table_mut()
            .context("OhMyKeymint [trust] is not a table")?;
        for key in TRUST_FIELDS {
            if let Some(value) = config.default_policy.get(*key) {
                trust.insert((*key).to_owned(), trust_value(key, value));
            }
        }
        write_table(sysroot, "config.toml", &main)
    }
}

fn read_table(sysroot: &Sysroot, file: &str) -> Result<Option<Table>> {
    let path = sysroot.path(format!("{OMK_CONFIG_DIR}/{file}"));
    match duck_core::fs::read_optional(&path)? {
        Some(raw) => Ok(Some(
            toml::from_str(&raw).with_context(|| format!("parse OhMyKeymint {file}"))?,
        )),
        None => Ok(None),
    }
}

fn write_table(sysroot: &Sysroot, file: &str, table: &Table) -> Result<()> {
    let path = sysroot.path(format!("{OMK_CONFIG_DIR}/{file}"));
    let body =
        toml::to_string_pretty(table).with_context(|| format!("serialize OhMyKeymint {file}"))?;
    write_bytes_preserving(&path, body.as_bytes())
}

fn scalar_to_string(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Integer(number) => number.to_string(),
        Value::Boolean(flag) => flag.to_string(),
        other => other.to_string(),
    }
}

/// `os_version` is stored as an integer when numeric; every other trust field is a string.
fn trust_value(key: &str, value: &str) -> Value {
    if key == "os_version"
        && let Ok(number) = value.parse::<i64>()
    {
        return Value::Integer(number);
    }
    Value::String(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{scalar_to_string, trust_value};
    use toml::Value;

    #[test]
    fn os_version_number_is_stored_as_integer() {
        assert_eq!(trust_value("os_version", "15"), Value::Integer(15));
        assert_eq!(
            trust_value("os_version", "auto"),
            Value::String("auto".into())
        );
        assert_eq!(trust_value("vb_key", "auto"), Value::String("auto".into()));
    }

    #[test]
    fn scalars_render_as_strings() {
        assert_eq!(scalar_to_string(&Value::Integer(15)), "15");
        assert_eq!(scalar_to_string(&Value::String("auto".into())), "auto");
    }
}
