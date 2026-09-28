//! Auto-target: on boot, add newly installed user apps to the target list.
//!
//! The first run after enabling records a baseline of installed user apps; later runs add
//! only apps that appeared since. This is the boot-time counterpart of the WebUI toggle.

use anyhow::Result;
use duck_core::Context;
use duck_platform::packages::{self, PackageFilter};
use serde::Serialize;

use crate::{
    model::{TargetEntry, TargetMode},
    service::active_adapter,
    state::State,
    targets::normalize_packages,
};

#[derive(Debug, Serialize)]
pub struct AutoApplyData {
    pub enabled: bool,
    pub added: Vec<String>,
    pub target_count: usize,
}

pub fn apply(ctx: &Context) -> Result<AutoApplyData> {
    let mut state = State::load(&ctx.paths, &ctx.sysroot)?;
    if !state.auto.enabled {
        return Ok(AutoApplyData {
            enabled: false,
            added: Vec::new(),
            target_count: 0,
        });
    }

    let user_packages = normalize_packages(packages::list(PackageFilter::User));

    // First run just records the baseline so pre-existing apps are not mass-added.
    if !state.auto.baseline_initialized {
        state.auto.known_user_apps = user_packages;
        state.auto.baseline_initialized = true;
        state.save(&ctx.paths)?;
        return Ok(AutoApplyData {
            enabled: true,
            added: Vec::new(),
            target_count: 0,
        });
    }

    let (detection, adapter) = active_adapter(&ctx.sysroot)?;
    let mut config = adapter.read(&ctx.sysroot)?;
    let known: std::collections::BTreeSet<&str> = state
        .auto
        .known_user_apps
        .iter()
        .map(String::as_str)
        .collect();
    let existing: std::collections::BTreeSet<String> = config
        .targets
        .iter()
        .map(|entry| entry.package_name.clone())
        .collect();

    let mut added = Vec::new();
    for package in &user_packages {
        if !known.contains(package.as_str()) && !existing.contains(package) {
            config.targets.push(TargetEntry {
                package_name: package.clone(),
                mode: TargetMode::Auto,
            });
            added.push(package.clone());
        }
    }

    if !added.is_empty() {
        let _ = detection;
        adapter.write(&ctx.sysroot, &config)?;
    }

    state.auto.known_user_apps = user_packages;
    state.save(&ctx.paths)?;

    Ok(AutoApplyData {
        enabled: true,
        target_count: config.targets.len(),
        added,
    })
}
