//! Summary of the module, root manager and device shown on the WebUI home screen.

use std::collections::BTreeMap;

use anyhow::{Context as _, Result};
use duck_core::{AppPaths, Sysroot};
use duck_platform::{
    exec,
    modules::parse_module_prop,
    props,
    root::{self, RootManagerKind},
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ModuleIdentity {
    pub id: String,
    pub name: String,
    pub version: String,
    pub version_code: u64,
    pub author: String,
    pub update_json: Option<String>,
    /// Version of the `duckd` binary, which ships inside the module.
    pub binary_version: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct RootManagerInfo {
    pub kind: RootManagerKind,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DeviceSummary {
    pub brand: String,
    pub model: String,
    pub device: String,
    pub android_release: String,
    pub sdk: String,
    pub security_patch: String,
    pub fingerprint: String,
    pub abi: String,
    pub kernel_release: Option<String>,
    /// `enforcing` or `permissive`; `None` when SELinux state is not readable.
    pub selinux: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemInfo {
    pub module: ModuleIdentity,
    pub root_manager: Option<RootManagerInfo>,
    pub device: DeviceSummary,
}

pub fn collect(paths: &AppPaths, sysroot: &Sysroot) -> Result<SystemInfo> {
    let module = module_identity(paths)?;
    let root_manager = root::detect(sysroot).map(|manager| RootManagerInfo {
        version: manager.version(),
        kind: manager.kind,
    });
    Ok(SystemInfo {
        module,
        root_manager,
        device: device_summary(sysroot, &props::all()),
    })
}

pub fn module_identity(paths: &AppPaths) -> Result<ModuleIdentity> {
    let prop_path = paths.root.join("module.prop");
    let raw = std::fs::read_to_string(&prop_path)
        .with_context(|| format!("read {}", prop_path.display()))?;
    let props = parse_module_prop(&raw);
    let text = |key: &str| props.get(key).cloned().unwrap_or_default();
    Ok(ModuleIdentity {
        id: text("id"),
        name: text("name"),
        version: text("version"),
        version_code: props
            .get("versionCode")
            .and_then(|value| value.parse().ok())
            .unwrap_or(0),
        author: text("author"),
        update_json: props.get("updateJson").cloned(),
        binary_version: env!("CARGO_PKG_VERSION"),
    })
}

fn device_summary(sysroot: &Sysroot, props: &BTreeMap<String, String>) -> DeviceSummary {
    let get = |key: &str| props.get(key).cloned().unwrap_or_default();
    DeviceSummary {
        brand: get("ro.product.brand"),
        model: get("ro.product.model"),
        device: get("ro.product.device"),
        android_release: get("ro.build.version.release"),
        sdk: get("ro.build.version.sdk"),
        security_patch: get("ro.build.version.security_patch"),
        fingerprint: get("ro.build.fingerprint"),
        abi: get("ro.product.cpu.abi"),
        kernel_release: kernel_release(sysroot),
        selinux: selinux_mode(sysroot),
    }
}

fn selinux_mode(sysroot: &Sysroot) -> Option<&'static str> {
    let enforcing = match read_trimmed(sysroot, "/sys/fs/selinux/enforce") {
        Some(value) => value != "0",
        // Unreadable from some domains, like osrelease; toybox getenforce asks the kernel.
        None => match exec::stdout_or_empty("getenforce", &[]).trim() {
            "Enforcing" => true,
            "Permissive" => false,
            _ => return None,
        },
    };
    Some(if enforcing { "enforcing" } else { "permissive" })
}

/// SELinux keeps some domains (e.g. `shell`) from reading `/proc/sys/kernel/osrelease`,
/// while uname(2) is always allowed.
fn kernel_release(sysroot: &Sysroot) -> Option<String> {
    read_trimmed(sysroot, "/proc/sys/kernel/osrelease").or_else(|| {
        let release = exec::stdout_or_empty("uname", &["-r"]);
        let release = release.trim();
        (!release.is_empty()).then(|| release.to_owned())
    })
}

fn read_trimmed(sysroot: &Sysroot, device_path: &str) -> Option<String> {
    std::fs::read_to_string(sysroot.path(device_path))
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, fs};

    use duck_core::{AppPaths, Sysroot};

    use super::{device_summary, module_identity};

    #[test]
    fn reads_module_prop_and_kernel_state() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("module");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("module.prop"),
            "id=duck-toolbox\nname=Duck ToolBox\nversion=v1.2.0\nversionCode=12\nupdateJson=https://example.com/u.json\n",
        )
        .unwrap();
        let identity = module_identity(&AppPaths::for_root(&root, &root)).unwrap();
        assert_eq!(identity.version_code, 12);
        assert_eq!(
            identity.update_json.as_deref(),
            Some("https://example.com/u.json")
        );

        let sysroot = Sysroot::new(dir.path().join("sys"));
        fs::create_dir_all(sysroot.path("/proc/sys/kernel")).unwrap();
        fs::create_dir_all(sysroot.path("/sys/fs/selinux")).unwrap();
        fs::write(
            sysroot.path("/proc/sys/kernel/osrelease"),
            "6.6.30-android15\n",
        )
        .unwrap();
        fs::write(sysroot.path("/sys/fs/selinux/enforce"), "1").unwrap();
        let props = BTreeMap::from([("ro.product.model".to_owned(), "Pixel 9".to_owned())]);

        let device = device_summary(&sysroot, &props);
        assert_eq!(device.model, "Pixel 9");
        assert_eq!(device.kernel_release.as_deref(), Some("6.6.30-android15"));
        assert_eq!(device.selinux, Some("enforcing"));
    }
}
