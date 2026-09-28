//! Picks which keystore backend to manage.
//!
//! Preference order matches Tricky Addon's runtime detection: an active Tricky Store wins,
//! then OhMyKeymint, then TEESimulator. The chosen backend selects the config adapter.

use duck_core::Sysroot;
use duck_platform::modules;

use crate::model::{Backend, BackendDetection};

/// `(backend, module id)` pairs in preference order.
const KNOWN_BACKENDS: &[(Backend, &str)] = &[
    (Backend::TrickyStore, "tricky_store"),
    (Backend::OhMyKeymint, "oh_my_keymint"),
    (Backend::TeeSimulator, "teesim"),
];

/// All installed backends, so the WebUI can report every one it found.
pub fn detect_all(sysroot: &Sysroot) -> Vec<BackendDetection> {
    KNOWN_BACKENDS
        .iter()
        .filter_map(|(backend, id)| detect_one(sysroot, *backend, id))
        .collect()
}

/// The backend Duck ToolBox will manage: the first active one, or the first installed one
/// if none are active.
pub fn detect_active(sysroot: &Sysroot) -> Option<BackendDetection> {
    let found = detect_all(sysroot);
    found
        .iter()
        .find(|detection| detection.active)
        .or_else(|| found.first())
        .cloned()
}

fn detect_one(sysroot: &Sysroot, backend: Backend, id: &str) -> Option<BackendDetection> {
    let module = modules::find(sysroot, id)?;
    let mut refined = backend;
    if backend == Backend::TrickyStore && !tricky_store_supports_ini(module.version_code) {
        refined = Backend::TrickyStoreLegacy;
    }

    let active = module.is_active();
    let version_code = module.version_code;
    Some(BackendDetection {
        backend: refined,
        identity: refined.identity(),
        module_id: module.id,
        module_dir: module.dir,
        name: module.name,
        version: module.version,
        version_code,
        active,
    })
}

/// Tricky Store adopted the `config.ini` format at versionCode 246; older builds use the
/// legacy `target.txt` layout.
fn tricky_store_supports_ini(version_code: Option<u64>) -> bool {
    version_code.is_some_and(|code| code >= 246)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use duck_core::Sysroot;

    use super::{detect_active, tricky_store_supports_ini};
    use crate::model::Backend;

    fn sysroot(name: &str) -> Sysroot {
        let root =
            std::env::temp_dir().join(format!("duck-ts-detect-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        Sysroot::new(root)
    }

    fn install(sysroot: &Sysroot, id: &str, version_code: u64, disabled: bool) {
        let dir = sysroot.path(format!("/data/adb/modules/{id}"));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("module.prop"),
            format!("id={id}\nname={id}\nversionCode={version_code}\n"),
        )
        .unwrap();
        if disabled {
            fs::write(dir.join("disable"), "").unwrap();
        }
    }

    #[test]
    fn version_code_selects_ini_or_legacy() {
        assert!(tricky_store_supports_ini(Some(246)));
        assert!(!tricky_store_supports_ini(Some(245)));
        assert!(!tricky_store_supports_ini(None));
    }

    #[test]
    fn active_backend_wins_over_disabled() {
        let sysroot = sysroot("active");
        install(&sysroot, "tricky_store", 245, true);
        install(&sysroot, "teesim", 400, false);

        let active = detect_active(&sysroot).unwrap();
        assert_eq!(active.backend, Backend::TeeSimulator);
        assert!(active.active);
    }

    #[test]
    fn falls_back_to_legacy_tricky_store() {
        let sysroot = sysroot("legacy");
        install(&sysroot, "tricky_store", 200, false);

        let active = detect_active(&sysroot).unwrap();
        assert_eq!(active.backend, Backend::TrickyStoreLegacy);
    }
}
