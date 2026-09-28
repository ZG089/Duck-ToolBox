//! Optional WebUI entry on the keystore module itself, as Tricky Addon provides.
//!
//! Managers show a WebUI button for any module with a `webroot` directory (KernelSU module
//! guide), so linking Duck ToolBox's `webroot` into the active keystore module gives that
//! module a button that opens this manager. Modules that ship their own WebUI (TEESimulator)
//! are left alone. On Magisk, which has no WebUI, `action.sh` is linked instead.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use duck_core::Context;
use duck_platform::root::{self, RootManagerKind};
use serde::Serialize;

use crate::{detect, error::TrickyError, state::State};

#[derive(Debug, Clone, Serialize)]
pub struct EntryStatus {
    pub enabled: bool,
    /// Module the entry is (or would be) attached to.
    pub module_id: Option<String>,
    pub linked: bool,
    /// The keystore module ships its own WebUI, so no entry can be attached.
    pub has_own_webui: bool,
}

pub fn status(ctx: &Context) -> Result<EntryStatus> {
    let state = State::load(&ctx.paths, &ctx.sysroot)?;
    let Some(target) = target(ctx) else {
        return Ok(EntryStatus {
            enabled: state.entry_enabled,
            module_id: None,
            linked: false,
            has_own_webui: false,
        });
    };
    Ok(EntryStatus {
        enabled: state.entry_enabled,
        linked: is_our_link(&target.link, &target.source),
        has_own_webui: target.link.exists() && !is_our_link(&target.link, &target.source),
        module_id: Some(target.module_id),
    })
}

pub fn set(ctx: &Context, enabled: bool) -> Result<EntryStatus> {
    let mut state = State::load(&ctx.paths, &ctx.sysroot)?;
    state.entry_enabled = enabled;
    if enabled {
        let target = target(ctx).ok_or(TrickyError::NoBackend)?;
        attach(&target)?;
    } else if let Some(target) = target(ctx) {
        detach(&target)?;
    }
    state.save(&ctx.paths)?;
    status(ctx)
}

/// Re-creates or removes the link to match the saved setting; run on boot, since a
/// keystore module update replaces its directory.
pub fn apply(ctx: &Context) -> Result<EntryStatus> {
    let state = State::load(&ctx.paths, &ctx.sysroot)?;
    if let Some(target) = target(ctx) {
        if state.entry_enabled {
            let _ = attach(&target);
        } else {
            detach(&target)?;
        }
    }
    status(ctx)
}

/// Removes every link this feature may have created, on uninstall.
pub fn remove_all(ctx: &Context) -> Result<()> {
    for detection in detect::detect_all(&ctx.sysroot) {
        for (name, source) in link_names(ctx) {
            let link = ctx.sysroot.path(format!("{}/{name}", detection.module_dir));
            if is_our_link(&link, &source) {
                std::fs::remove_file(&link)
                    .with_context(|| format!("remove {}", link.display()))?;
            }
        }
    }
    Ok(())
}

struct Target {
    module_id: String,
    link: PathBuf,
    source: PathBuf,
}

fn link_names(ctx: &Context) -> [(&'static str, PathBuf); 2] {
    [
        ("webroot", ctx.paths.root.join("webroot")),
        ("action.sh", ctx.paths.root.join("action.sh")),
    ]
}

fn target(ctx: &Context) -> Option<Target> {
    let detection = detect::detect_active(&ctx.sysroot)?;
    let magisk =
        root::detect(&ctx.sysroot).is_some_and(|manager| manager.kind == RootManagerKind::Magisk);
    let (name, source) = if magisk {
        ("action.sh", ctx.paths.root.join("action.sh"))
    } else {
        ("webroot", ctx.paths.root.join("webroot"))
    };
    Some(Target {
        module_id: detection.module_id,
        link: ctx.sysroot.path(format!("{}/{name}", detection.module_dir)),
        source,
    })
}

fn is_our_link(link: &Path, source: &Path) -> bool {
    std::fs::read_link(link).is_ok_and(|points_to| points_to == source)
}

fn attach(target: &Target) -> Result<()> {
    if is_our_link(&target.link, &target.source) {
        return Ok(());
    }
    if target.link.exists() || target.link.is_symlink() {
        return Err(TrickyError::BackendRule(format!(
            "{} already provides its own entry",
            target.module_id
        ))
        .into());
    }
    symlink(&target.source, &target.link).with_context(|| format!("link {}", target.link.display()))
}

fn detach(target: &Target) -> Result<()> {
    if is_our_link(&target.link, &target.source) {
        std::fs::remove_file(&target.link)
            .with_context(|| format!("remove {}", target.link.display()))?;
    }
    Ok(())
}

#[cfg(unix)]
fn symlink(source: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(source, link)
}

#[cfg(not(unix))]
fn symlink(_source: &Path, _link: &Path) -> std::io::Result<()> {
    Err(std::io::Error::other("symlinks require a Unix system"))
}

#[cfg(all(test, unix))]
mod tests {
    use std::fs;

    use duck_core::{AppPaths, Context, Sysroot};

    use super::{apply, remove_all, set, status};

    fn context(root: &std::path::Path) -> Context {
        let module = root.join("module");
        fs::create_dir_all(module.join("webroot")).unwrap();
        fs::write(module.join("module.prop"), "id=duck-toolbox\n").unwrap();
        Context {
            paths: AppPaths::for_root(&module, &root.join("data")),
            sysroot: Sysroot::new(root.join("sys")),
        }
    }

    fn install(ctx: &Context, id: &str) -> std::path::PathBuf {
        let dir = ctx.sysroot.path(format!("/data/adb/modules/{id}"));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("module.prop"),
            format!("id={id}\nversionCode=246\n"),
        )
        .unwrap();
        dir
    }

    #[test]
    fn links_and_unlinks_the_webroot() {
        let root = tempfile::tempdir().unwrap();
        let ctx = context(root.path());
        let dir = install(&ctx, "tricky_store");

        let enabled = set(&ctx, true).unwrap();
        assert!(enabled.linked && enabled.enabled);
        assert_eq!(
            fs::read_link(dir.join("webroot")).unwrap(),
            ctx.paths.root.join("webroot")
        );

        fs::remove_file(dir.join("webroot")).unwrap();
        assert!(apply(&ctx).unwrap().linked, "boot re-creates the link");

        let disabled = set(&ctx, false).unwrap();
        assert!(!disabled.linked && !dir.join("webroot").exists());
    }

    #[test]
    fn leaves_a_module_with_its_own_webui_alone() {
        let root = tempfile::tempdir().unwrap();
        let ctx = context(root.path());
        let dir = install(&ctx, "tricky_store");
        fs::create_dir_all(dir.join("webroot")).unwrap();

        assert!(set(&ctx, true).is_err());
        assert!(status(&ctx).unwrap().has_own_webui);
        remove_all(&ctx).unwrap();
        assert!(
            dir.join("webroot").is_dir(),
            "a real directory is never removed"
        );
    }
}
