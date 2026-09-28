//! Duck ToolBox's own settings for this feature, stored in `var/tricky-store.toml`.
//!
//! Tricky Addon kept these in WebView `localStorage`, which is lost when the manager is
//! reinstalled; the KernelSU WebUI guide recommends persisting to a directory instead.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use duck_core::{AppPaths, Sysroot, fs::write_string_atomic};
use serde::{Deserialize, Serialize};

use crate::adapters::TS_CONFIG_DIR;

const STATE_FILE: &str = "tricky-store.toml";
const LEGACY_AUTO_FILE: &str = "tricky-store-auto-target.toml";
const STATE_VERSION: u32 = 1;

/// System apps shown in the list by default, matching Tricky Addon.
pub(crate) const DEFAULT_SYSTEM_APPS: &[&str] = &[
    "com.google.android.gms",
    "com.android.vending",
    "com.oplus.deepthinker",
    "com.heytap.speechassist",
    "com.coloros.sceneservice",
];

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoTargetState {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub baseline_initialized: bool,
    #[serde(default)]
    pub known_user_apps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyboxProvider {
    pub name: String,
    #[serde(alias = "link")]
    pub url: String,
    /// Decode pipeline in Tricky Addon's shell syntax, e.g. `xxd -r -p | base64 -d`.
    #[serde(default, alias = "script")]
    pub decode: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    #[serde(default = "state_version")]
    pub version: u32,
    #[serde(default)]
    pub system_apps: Vec<String>,
    #[serde(default)]
    pub auto: AutoTargetState,
    #[serde(default = "default_providers")]
    pub providers: Vec<KeyboxProvider>,
    /// Whether the keystore module should carry a WebUI entry for this manager.
    #[serde(default)]
    pub entry_enabled: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            version: STATE_VERSION,
            system_apps: DEFAULT_SYSTEM_APPS
                .iter()
                .map(|app| (*app).to_owned())
                .collect(),
            auto: AutoTargetState::default(),
            providers: default_providers(),
            entry_enabled: false,
        }
    }
}

fn state_version() -> u32 {
    STATE_VERSION
}

/// The provider Tricky Addon ships by default.
pub(crate) fn default_providers() -> Vec<KeyboxProvider> {
    vec![KeyboxProvider {
        name: "Addon".into(),
        url: "https://raw.githubusercontent.com/KOWX712/Tricky-Addon-Update-Target-List/keybox/.extra"
            .into(),
        decode: "xxd -r -p | base64 -d".into(),
    }]
}

impl State {
    pub fn path(paths: &AppPaths) -> PathBuf {
        paths.var_dir.join(STATE_FILE)
    }

    /// Loads the state, migrating settings written by earlier Duck ToolBox releases.
    pub fn load(paths: &AppPaths, sysroot: &Sysroot) -> Result<Self> {
        let path = Self::path(paths);
        if let Some(raw) = duck_core::fs::read_optional(&path)? {
            let mut state: Self =
                toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
            state.normalize();
            return Ok(state);
        }

        let mut state = Self::default();
        if let Some(auto) = read_legacy_auto(&paths.var_dir.join(LEGACY_AUTO_FILE))? {
            state.auto = auto;
        }
        let legacy_system_apps = sysroot.path(format!("{TS_CONFIG_DIR}/system_app"));
        if let Some(raw) = duck_core::fs::read_optional(&legacy_system_apps)? {
            state.system_apps.extend(
                raw.lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .map(str::to_owned),
            );
        }
        state.normalize();
        Ok(state)
    }

    pub fn save(&self, paths: &AppPaths) -> Result<()> {
        paths.ensure_runtime_dirs()?;
        let body = toml::to_string_pretty(self).context("serialize tricky-store state")?;
        write_string_atomic(&Self::path(paths), &body)
    }

    fn normalize(&mut self) {
        self.version = STATE_VERSION;
        self.system_apps =
            crate::targets::normalize_packages(std::mem::take(&mut self.system_apps));
        self.auto.known_user_apps =
            crate::targets::normalize_packages(std::mem::take(&mut self.auto.known_user_apps));
        if !self.auto.enabled {
            self.auto.baseline_initialized = false;
            self.auto.known_user_apps.clear();
        }
    }
}

fn read_legacy_auto(path: &Path) -> Result<Option<AutoTargetState>> {
    let Some(raw) = duck_core::fs::read_optional(path)? else {
        return Ok(None);
    };
    Ok(toml::from_str(&raw).ok())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use duck_core::{AppPaths, Sysroot};

    use super::{DEFAULT_SYSTEM_APPS, State};

    fn fixture(name: &str) -> (AppPaths, Sysroot) {
        let root =
            std::env::temp_dir().join(format!("duck-ts-state-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("bin")).unwrap();
        fs::write(root.join("module.prop"), "id=duck-toolbox\n").unwrap();
        let paths = AppPaths::for_root(&root, &root.join("data"));
        (paths, Sysroot::new(root.join("sys")))
    }

    #[test]
    fn defaults_include_play_services() {
        let (paths, sysroot) = fixture("defaults");
        let state = State::load(&paths, &sysroot).unwrap();

        assert!(
            state
                .system_apps
                .iter()
                .any(|app| app == DEFAULT_SYSTEM_APPS[0])
        );
        assert_eq!(state.providers.len(), 1);
    }

    #[test]
    fn migrates_legacy_auto_target_and_system_app_files() {
        let (paths, sysroot) = fixture("migrate");
        fs::create_dir_all(&paths.var_dir).unwrap();
        fs::write(
            paths.var_dir.join("tricky-store-auto-target.toml"),
            "version = 1\nenabled = true\nbaseline_initialized = true\nknown_user_apps = [\"com.old\"]\n",
        )
        .unwrap();
        let legacy = sysroot.path("/data/adb/tricky_store/system_app");
        fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        fs::write(&legacy, "com.extra.system\n").unwrap();

        let state = State::load(&paths, &sysroot).unwrap();

        assert!(state.auto.enabled);
        assert_eq!(state.auto.known_user_apps, vec!["com.old"]);
        assert!(
            state
                .system_apps
                .iter()
                .any(|app| app == "com.extra.system")
        );
    }

    #[test]
    fn round_trips_through_disk() {
        let (paths, sysroot) = fixture("roundtrip");
        let mut state = State::load(&paths, &sysroot).unwrap();
        state.system_apps = vec!["com.b".into(), "com.a".into(), "com.a".into()];
        state.save(&paths).unwrap();

        let reloaded = State::load(&paths, &sysroot).unwrap();
        assert_eq!(reloaded.system_apps, vec!["com.a", "com.b"]);
    }
}
