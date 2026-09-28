//! Root implementation detection from the on-disk layout each manager documents.

use duck_core::Sysroot;
use serde::Serialize;

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

#[cfg(test)]
mod tests {
    use std::fs;

    use duck_core::Sysroot;

    use super::{RootManagerKind, detect};

    #[test]
    fn prefers_kernelsu_over_magisk_leftovers() {
        let root = std::env::temp_dir().join(format!("duck-root-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let sysroot = Sysroot::new(&root);
        assert!(detect(&sysroot).is_none());

        fs::create_dir_all(sysroot.path("/data/adb/magisk")).unwrap();
        assert_eq!(detect(&sysroot).unwrap().kind, RootManagerKind::Magisk);

        fs::create_dir_all(sysroot.path("/data/adb/ksu")).unwrap();
        assert_eq!(detect(&sysroot).unwrap().kind, RootManagerKind::KernelSu);
    }
}
