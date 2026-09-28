//! Fails when any tracked source file exceeds the line limit.
//!
//! Large files resist the modular, easy-to-extend structure the project targets, so the
//! limit is enforced in CI. Only human-authored sources count; generated files, lockfiles,
//! vendored assets and localization bundles are exempt.

use std::{path::Path, process::Command};

use anyhow::{Context, Result, bail};

const MAX_LINES: usize = 600;

/// Extensions treated as source code for the purpose of the limit.
const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "ts", "tsx", "js", "vue", "css", "scss", "sh", "ps1", "py", "yml", "yaml",
];

/// Path fragments that are exempt: generated bundles, lockfiles and vendored data.
const EXEMPT_FRAGMENTS: &[&str] = &[
    "pnpm-lock.yaml",
    "Cargo.lock",
    "/assets/",
    "/locales/",
    "webroot/",
];

pub fn run(repo_root: &Path) -> Result<()> {
    let mut offenders = Vec::new();

    for path in tracked_files(repo_root)? {
        if !is_source(&path) || is_exempt(&path) {
            continue;
        }
        let full = repo_root.join(&path);
        let Ok(contents) = std::fs::read_to_string(&full) else {
            continue; // Binary or unreadable files are not source.
        };
        let lines = contents.lines().count();
        if lines > MAX_LINES {
            offenders.push((path, lines));
        }
    }

    if offenders.is_empty() {
        println!("line-limit: all source files are within {MAX_LINES} lines");
        return Ok(());
    }

    offenders.sort_by_key(|(_, lines)| std::cmp::Reverse(*lines));
    eprintln!(
        "line-limit: {} file(s) exceed {MAX_LINES} lines:",
        offenders.len()
    );
    for (path, lines) in &offenders {
        eprintln!("  {lines:>5}  {path}");
    }
    bail!("split these files so every source file stays within {MAX_LINES} lines");
}

fn tracked_files(repo_root: &Path) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["ls-files"])
        .current_dir(repo_root)
        .output()
        .context("run `git ls-files`")?;
    if !output.status.success() {
        bail!(
            "`git ls-files` failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect())
}

fn is_source(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|ext| SOURCE_EXTENSIONS.contains(&ext))
}

fn is_exempt(path: &str) -> bool {
    EXEMPT_FRAGMENTS
        .iter()
        .any(|fragment| path.contains(fragment))
}

#[cfg(test)]
mod tests {
    use super::{is_exempt, is_source};

    #[test]
    fn recognizes_source_extensions() {
        assert!(is_source("crates/duck-core/src/lib.rs"));
        assert!(is_source("ui/src/App.vue"));
        assert!(!is_source("README.md"));
        assert!(!is_source("assets/logo.png"));
    }

    #[test]
    fn exempts_generated_and_vendored_files() {
        assert!(is_exempt("ui/pnpm-lock.yaml"));
        assert!(is_exempt("Cargo.lock"));
        assert!(is_exempt("crates/duck-tricky-store/assets/aosp-keybox.xml"));
        assert!(!is_exempt("crates/duck-core/src/lib.rs"));
    }
}
