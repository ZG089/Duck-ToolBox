//! Backend-agnostic data model shared by every keystore adapter and the WebUI.
//!
//! A "policy" is a flat `key -> string` map whose editable shape is described by a
//! [`PolicySchema`]. The WebUI renders fields from the schema, so adding a field to one
//! backend (or a whole new backend) needs no UI change.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Which upstream keystore-spoofing module a config belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Backend {
    /// 5ec1cff Tricky Store (and forks) using the `config.ini` format.
    TrickyStore,
    /// Classic Tricky Store using `target.txt` + `security_patch.txt`.
    TrickyStoreLegacy,
    /// JingMatrix TEESimulator (`config.json`).
    TeeSimulator,
    /// qwq233 OhMyKeymint (`config.toml` + `injector.toml`).
    OhMyKeymint,
}

impl Backend {
    /// Short tag shown in the WebUI, matching Tricky Addon's working-mode labels.
    pub fn identity(self) -> &'static str {
        match self {
            Self::TrickyStore => "TS",
            Self::TrickyStoreLegacy => "TS-L",
            Self::TeeSimulator => "TEES",
            Self::OhMyKeymint => "OMK",
        }
    }
}

/// Per-app attestation mode. Only Tricky Store honours generate/hack; other backends treat
/// every target as [`TargetMode::Auto`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TargetMode {
    #[default]
    Auto,
    /// Force certificate generation (`pkg!`).
    Generate,
    /// Force leaf hack (`pkg?`).
    Hack,
}

impl TargetMode {
    pub fn marker(self) -> &'static str {
        match self {
            Self::Auto => "",
            Self::Generate => "!",
            Self::Hack => "?",
        }
    }

    /// Splits a `target.txt` line into its package name and mode.
    pub fn split(line: &str) -> (String, Self) {
        let line = line.trim();
        if let Some(name) = line.strip_suffix('!') {
            (name.trim().to_owned(), Self::Generate)
        } else if let Some(name) = line.strip_suffix('?') {
            (name.trim().to_owned(), Self::Hack)
        } else {
            (line.to_owned(), Self::Auto)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetEntry {
    pub package_name: String,
    #[serde(default)]
    pub mode: TargetMode,
}

pub type Policy = BTreeMap<String, String>;

/// One editable policy field, rendered generically by the WebUI.
#[derive(Debug, Clone, Serialize)]
pub struct PolicyField {
    pub key: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    /// Suggested values offered as quick-fill chips (e.g. `auto`, `prop`, `no`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u32>,
    #[serde(default)]
    pub multiline: bool,
    /// Human-readable description of accepted values, shown as validation help.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// Server-side check for submitted values; the WebUI shows `hint` when it fails.
    #[serde(skip)]
    pub validator: Option<fn(&str) -> bool>,
}

impl PolicyField {
    pub fn new(key: &str, label: &str) -> Self {
        Self {
            key: key.to_owned(),
            label: label.to_owned(),
            placeholder: None,
            options: Vec::new(),
            max_length: None,
            multiline: false,
            hint: None,
            validator: None,
        }
    }

    pub fn validate(mut self, validator: fn(&str) -> bool) -> Self {
        self.validator = Some(validator);
        self
    }

    pub fn placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = Some(placeholder.to_owned());
        self
    }

    pub fn options(mut self, options: &[&str]) -> Self {
        self.options = options.iter().map(|value| (*value).to_owned()).collect();
        self
    }

    pub fn max_length(mut self, max_length: u32) -> Self {
        self.max_length = Some(max_length);
        self
    }

    pub fn multiline(mut self) -> Self {
        self.multiline = true;
        self
    }

    pub fn hint(mut self, hint: &str) -> Self {
        self.hint = Some(hint.to_owned());
        self
    }
}

/// Capabilities and editable fields a backend exposes to the WebUI.
#[derive(Debug, Clone, Serialize)]
pub struct PolicySchema {
    pub supports_app_mode: bool,
    pub supports_per_app_policy: bool,
    pub default_policy: Vec<PolicyField>,
}

/// A backend's full configuration, decoded into the shared shape.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigData {
    #[serde(default)]
    pub targets: Vec<TargetEntry>,
    #[serde(default)]
    pub default_policy: Policy,
    /// Per-app policy overrides (Tricky Store `config.ini` only).
    #[serde(default)]
    pub per_app_policy: BTreeMap<String, Policy>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PackageEntry {
    pub package_name: String,
    pub system: bool,
    pub selected: bool,
    pub mode: TargetMode,
    /// Whether the app is tracked as an extra system app to keep visible.
    pub tracked_system: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyboxStatus {
    pub path: String,
    pub exists: bool,
    pub size: u64,
    pub modified_unix: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackendDetection {
    pub backend: Backend,
    pub identity: &'static str,
    pub module_id: String,
    pub module_dir: String,
    pub name: Option<String>,
    pub version: Option<String>,
    pub version_code: Option<u64>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PropStatus {
    pub prop_handler_enabled: bool,
    pub boot_hash: Option<String>,
}

/// Request written by the WebUI's "Save" action.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SaveRequest {
    #[serde(default)]
    pub targets: Vec<TargetEntry>,
    #[serde(default)]
    pub default_policy: Policy,
    #[serde(default)]
    pub per_app_policy: BTreeMap<String, Policy>,
    #[serde(default)]
    pub system_apps: Vec<String>,
    #[serde(default)]
    pub auto_add_new_apps: bool,
}
