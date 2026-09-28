//! Read-only view of installed root-manager modules.
//!
//! Layout and status flags follow the KernelSU module guide (shared with Magisk and
//! APatch): `/data/adb/modules/<id>/module.prop` plus the `disable`, `remove` and `update`
//! marker files.

use std::{collections::BTreeMap, fs, path::PathBuf};

use duck_core::Sysroot;
use serde::Serialize;

pub const MODULES_DIR: &str = "/data/adb/modules";
pub const MODULES_UPDATE_DIR: &str = "/data/adb/modules_update";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ModuleInfo {
    pub id: String,
    /// Device path of the module directory.
    pub dir: String,
    pub name: Option<String>,
    pub version: Option<String>,
    pub version_code: Option<u64>,
    pub author: Option<String>,
    pub enabled: bool,
    pub remove_pending: bool,
    pub update_pending: bool,
    #[serde(skip)]
    pub props: BTreeMap<String, String>,
    #[serde(skip)]
    pub host_dir: PathBuf,
}

impl ModuleInfo {
    /// A module is active when it is enabled and not scheduled for removal.
    pub fn is_active(&self) -> bool {
        self.enabled && !self.remove_pending
    }
}

/// Looks up a module by id. Returns `None` when the directory or its `module.prop` is
/// missing, which is how managers decide a directory is not a module.
pub fn find(sysroot: &Sysroot, id: &str) -> Option<ModuleInfo> {
    let device_dir = format!("{MODULES_DIR}/{id}");
    let host_dir = sysroot.path(&device_dir);
    let content = fs::read_to_string(host_dir.join("module.prop")).ok()?;
    let props = parse_module_prop(&content);

    Some(ModuleInfo {
        id: props.get("id").cloned().unwrap_or_else(|| id.to_owned()),
        dir: device_dir,
        name: props.get("name").cloned(),
        version: props.get("version").cloned(),
        version_code: props
            .get("versionCode")
            .and_then(|value| value.trim().parse().ok()),
        author: props.get("author").cloned(),
        enabled: !host_dir.join("disable").exists(),
        remove_pending: host_dir.join("remove").exists(),
        update_pending: host_dir.join("update").exists()
            || sysroot.path(format!("{MODULES_UPDATE_DIR}/{id}")).is_dir(),
        props,
        host_dir,
    })
}

/// Parses `module.prop`: one `key=value` pair per line, `#` comments, UNIX line endings
/// (CR is tolerated). Later duplicates win, matching how managers read the file.
pub fn parse_module_prop(content: &str) -> BTreeMap<String, String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
        .filter(|(key, _)| !key.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use duck_core::Sysroot;

    use super::{find, parse_module_prop};

    fn sysroot(name: &str) -> Sysroot {
        let root = std::env::temp_dir().join(format!(
            "duck-platform-modules-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        Sysroot::new(root)
    }

    #[test]
    fn parses_module_prop_like_managers() {
        let props = parse_module_prop(
            "id=tricky_store\r\nname=Tricky Store\n# comment\nversion=v1.4.1 (245)\nversionCode=245\ndescription=a=b\n",
        );

        assert_eq!(props["id"], "tricky_store");
        assert_eq!(props["version"], "v1.4.1 (245)");
        assert_eq!(props["description"], "a=b");
    }

    #[test]
    fn find_reports_flags() {
        let sysroot = sysroot("flags");
        let dir = sysroot.path("/data/adb/modules/teesim");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("module.prop"),
            "id=teesim\nname=TEESimulator\nversionCode=400\n",
        )
        .unwrap();
        fs::write(dir.join("disable"), "").unwrap();

        let module = find(&sysroot, "teesim").unwrap();

        assert_eq!(module.dir, "/data/adb/modules/teesim");
        assert_eq!(module.version_code, Some(400));
        assert!(!module.enabled);
        assert!(!module.is_active());
        assert!(find(&sysroot, "missing").is_none());
    }
}
