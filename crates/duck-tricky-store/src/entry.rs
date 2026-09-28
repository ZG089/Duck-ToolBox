//! Optional WebUI entry on the keystore module itself, as Tricky Addon provides.
//!
//! Managers show a WebUI button for any module with a `webroot` directory (KernelSU module
//! guide), so linking Duck ToolBox's `webroot` into the active keystore module gives that
//! module a button that opens this manager. Modules that ship their own WebUI (TEESimulator)
//! are left alone. Magisk has no WebUI button, so there `action.sh` is linked as well: it
//! opens the keystore module's (linked) WebUI in a standalone WebUI app.

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
    let webroot = &target.links[0];
    Ok(EntryStatus {
        enabled: state.entry_enabled,
        linked: target.links.iter().all(Link::is_ours),
        has_own_webui: webroot.path.exists() && !webroot.is_ours(),
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

/// Re-creates or removes the links to match the saved setting; run on boot, since a
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
        for name in LINKED {
            let link = link(ctx, &detection.module_dir, name);
            if link.is_ours() {
                std::fs::remove_file(&link.path)
                    .with_context(|| format!("remove {}", link.path.display()))?;
            }
        }
    }
    Ok(())
}

/// Everything that can be linked; the WebUI comes first.
const LINKED: [&str; 2] = ["webroot", "action.sh"];

struct Link {
    path: PathBuf,
    source: PathBuf,
}

impl Link {
    fn is_ours(&self) -> bool {
        std::fs::read_link(&self.path).is_ok_and(|points_to| points_to == self.source)
    }
}

struct Target {
    module_id: String,
    links: Vec<Link>,
}

fn link(ctx: &Context, module_dir: &str, name: &str) -> Link {
    Link {
        path: ctx.sysroot.path(format!("{module_dir}/{name}")),
        source: ctx.paths.root.join(name),
    }
}

fn target(ctx: &Context) -> Option<Target> {
    let detection = detect::detect_active(&ctx.sysroot)?;
    let magisk =
        root::detect(&ctx.sysroot).is_some_and(|manager| manager.kind == RootManagerKind::Magisk);
    let names = if magisk { &LINKED[..] } else { &LINKED[..1] };
    Some(Target {
        links: names
            .iter()
            .map(|name| link(ctx, &detection.module_dir, name))
            .collect(),
        module_id: detection.module_id,
    })
}

fn attach(target: &Target) -> Result<()> {
    let (webroot, extras) = target
        .links
        .split_first()
        .expect("the WebUI is always linked");
    if !webroot.is_ours() {
        if webroot.path.exists() || webroot.path.is_symlink() {
            return Err(TrickyError::BackendRule(format!(
                "{} already provides its own entry",
                target.module_id
            ))
            .into());
        }
        create(webroot)?;
    }
    // A keystore module's own action button is never replaced.
    for extra in extras {
        if !extra.is_ours() && !extra.path.exists() && !extra.path.is_symlink() {
            create(extra)?;
        }
    }
    Ok(())
}

fn create(link: &Link) -> Result<()> {
    symlink(&link.source, &link.path).with_context(|| format!("link {}", link.path.display()))
}

fn detach(target: &Target) -> Result<()> {
    for link in &target.links {
        if link.is_ours() {
            std::fs::remove_file(&link.path)
                .with_context(|| format!("remove {}", link.path.display()))?;
        }
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
    fn links_the_action_too_on_magisk_but_keeps_an_existing_one() {
        let root = tempfile::tempdir().unwrap();
        let ctx = context(root.path());
        fs::write(ctx.paths.root.join("action.sh"), "#!/system/bin/sh\n").unwrap();
        fs::create_dir_all(ctx.sysroot.path("/data/adb/magisk")).unwrap();
        let dir = install(&ctx, "tricky_store");

        assert!(set(&ctx, true).unwrap().linked);
        assert_eq!(
            fs::read_link(dir.join("action.sh")).unwrap(),
            ctx.paths.root.join("action.sh")
        );
        set(&ctx, false).unwrap();
        assert!(!dir.join("action.sh").exists() && !dir.join("webroot").exists());

        fs::write(dir.join("action.sh"), "own action\n").unwrap();
        assert!(set(&ctx, true).is_ok());
        assert_eq!(
            fs::read_to_string(dir.join("action.sh")).unwrap(),
            "own action\n"
        );
        remove_all(&ctx).unwrap();
        assert!(dir.join("action.sh").is_file() && !dir.join("webroot").exists());
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
