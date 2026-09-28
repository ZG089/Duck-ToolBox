//! Root implementation detection and the module operations each manager's CLI documents.
//!
//! KernelSU (`ksud module ...`, see `userspace/ksud/src/cli.rs`) and APatch (`apd module ...`)
//! share one command shape; Magisk only offers `--install-module`, so its uninstall uses the
//! `remove` marker file that every manager honours on the next boot.

use std::fs;

use anyhow::{Context as _, Result, bail};
use duck_core::Sysroot;
use serde::Serialize;

use crate::{exec, modules::MODULES_DIR};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RootManagerKind {
    /// KernelSU and its forks (KernelSU Next, SukiSU) share `/data/adb/ksud`.
    KernelSu,
    Apatch,
    Magisk,
}

#[derive(Debug, Clone, Serialize)]
pub struct RootManager {
    pub kind: RootManagerKind,
    /// Directory holding the manager's `busybox` and `resetprop`.
    pub bin_dir: &'static str,
}

/// KernelSU wins when leftovers of several managers exist, since its daemon is the one
/// actually running modules on a KernelSU kernel.
pub fn detect(sysroot: &Sysroot) -> Option<RootManager> {
    let exists = |path: &str| sysroot.path(path).exists();

    if exists("/data/adb/ksud") || exists("/data/adb/ksu") {
        Some(RootManager {
            kind: RootManagerKind::KernelSu,
            bin_dir: "/data/adb/ksu/bin",
        })
    } else if exists("/data/adb/apd") || exists("/data/adb/ap") {
        Some(RootManager {
            kind: RootManagerKind::Apatch,
            bin_dir: "/data/adb/ap/bin",
        })
    } else if exists("/data/adb/magisk") {
        Some(RootManager {
            kind: RootManagerKind::Magisk,
            bin_dir: "/data/adb/magisk",
        })
    } else {
        None
    }
}

impl RootManager {
    /// The manager's own CLI binary, falling back to a `PATH` lookup.
    pub fn cli(&self) -> String {
        match self.kind {
            RootManagerKind::KernelSu => exec::find_tool("ksud", &["/data/adb/ksud"]),
            RootManagerKind::Apatch => exec::find_tool("apd", &["/data/adb/apd"]),
            RootManagerKind::Magisk => magisk_cli(),
        }
    }

    /// Version string reported by the manager CLI (`ksud -V`, `apd -V`, `magisk -v`).
    pub fn version(&self) -> Option<String> {
        let flag = if self.kind == RootManagerKind::Magisk {
            "-v"
        } else {
            "-V"
        };
        let output = exec::stdout_or_empty(&self.cli(), &[flag]);
        let version = output.lines().next()?.trim();
        (!version.is_empty()).then(|| version.to_owned())
    }

    /// Installs a module zip the same way the manager app does. Returns the CLI output.
    pub fn install_module(&self, zip: &str) -> Result<String> {
        let cli = self.cli();
        let args: &[&str] = match self.kind {
            RootManagerKind::KernelSu | RootManagerKind::Apatch => &["module", "install", zip],
            RootManagerKind::Magisk => &["--install-module", zip],
        };
        run_reporting(&cli, args)
    }

    /// Marks a module for removal on the next boot.
    pub fn uninstall_module(&self, sysroot: &Sysroot, id: &str) -> Result<()> {
        match self.kind {
            RootManagerKind::KernelSu | RootManagerKind::Apatch => {
                run_reporting(&self.cli(), &["module", "uninstall", id]).map(|_| ())
            }
            RootManagerKind::Magisk => mark_for_removal(sysroot, id),
        }
    }

    /// Magisk DenyList packages (`magisk --denylist ls` prints `package|process` lines).
    /// Other managers have no DenyList, so this is empty for them.
    pub fn denylist(&self) -> Vec<String> {
        if self.kind != RootManagerKind::Magisk {
            return Vec::new();
        }
        parse_denylist(&exec::stdout_or_empty(&self.cli(), &["--denylist", "ls"]))
    }

    /// Replaces the description the KernelSU manager shows for module `id`, through the
    /// documented `override.description` module config key. Other managers have no such
    /// mechanism, so this is a no-op for them.
    pub fn set_module_description(&self, id: &str, description: &str) -> Result<()> {
        if self.kind != RootManagerKind::KernelSu {
            return Ok(());
        }
        let output = std::process::Command::new(self.cli())
            .args([
                "module",
                "config",
                "set",
                "override.description",
                description,
            ])
            .env("KSU_MODULE", id)
            .stdin(std::process::Stdio::null())
            .output()
            .context("run ksud module config")?;
        if !output.status.success() {
            bail!(
                "ksud module config set failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Ok(())
    }
}

fn magisk_cli() -> String {
    exec::find_tool(
        "magisk",
        &[
            "/data/adb/magisk/magisk",
            "/debug_ramdisk/magisk",
            "/sbin/magisk",
        ],
    )
}

fn run_reporting(program: &str, args: &[&str]) -> Result<String> {
    let output = exec::run(program, args)?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if !output.status.success() {
        bail!("`{program} {}` failed: {}", args.join(" "), combined.trim());
    }
    Ok(combined.trim().to_owned())
}

fn mark_for_removal(sysroot: &Sysroot, id: &str) -> Result<()> {
    let dir = sysroot.path(format!("{MODULES_DIR}/{id}"));
    if !dir.is_dir() {
        bail!("module {id} is not installed");
    }
    fs::write(dir.join("remove"), b"").with_context(|| format!("mark {id} for removal"))
}

pub fn parse_denylist(output: &str) -> Vec<String> {
    let mut packages: Vec<String> = output
        .lines()
        .filter(|line| !line.contains("isolated"))
        .filter_map(|line| line.split('|').next())
        .map(str::trim)
        .filter(|package| !package.is_empty())
        .map(str::to_owned)
        .collect();
    packages.sort();
    packages.dedup();
    packages
}

#[cfg(test)]
mod tests {
    use std::fs;

    use duck_core::Sysroot;

    use super::{RootManager, RootManagerKind, detect, parse_denylist};

    fn sysroot(name: &str) -> Sysroot {
        let root = std::env::temp_dir().join(format!("duck-root-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        Sysroot::new(root)
    }

    #[test]
    fn prefers_kernelsu_over_magisk_leftovers() {
        let sysroot = sysroot("detect");
        assert!(detect(&sysroot).is_none());

        fs::create_dir_all(sysroot.path("/data/adb/magisk")).unwrap();
        assert_eq!(detect(&sysroot).unwrap().kind, RootManagerKind::Magisk);

        fs::create_dir_all(sysroot.path("/data/adb/ksu")).unwrap();
        assert_eq!(detect(&sysroot).unwrap().kind, RootManagerKind::KernelSu);
    }

    #[test]
    fn magisk_uninstall_uses_the_remove_marker() {
        let sysroot = sysroot("magisk-remove");
        let manager = RootManager {
            kind: RootManagerKind::Magisk,
            bin_dir: "/data/adb/magisk",
        };
        assert!(manager.uninstall_module(&sysroot, "duck-toolbox").is_err());

        let dir = sysroot.path("/data/adb/modules/duck-toolbox");
        fs::create_dir_all(&dir).unwrap();
        manager.uninstall_module(&sysroot, "duck-toolbox").unwrap();
        assert!(dir.join("remove").is_file());
    }

    #[test]
    fn denylist_keeps_packages_and_skips_isolated() {
        let parsed = parse_denylist(
            "com.a|com.a\ncom.a|com.a:remote\ncom.b|com.b\ncom.c|isolated_process\n",
        );
        assert_eq!(parsed, vec!["com.a", "com.b"]);
    }

    #[test]
    fn description_override_is_kernelsu_only() {
        let magisk = RootManager {
            kind: RootManagerKind::Magisk,
            bin_dir: "/data/adb/magisk",
        };
        magisk
            .set_module_description("duck-toolbox", "ignored")
            .unwrap();
    }
}
