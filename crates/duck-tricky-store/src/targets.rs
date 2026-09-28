//! Target-list normalization shared by every adapter.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::{TargetEntry, TargetMode};

/// Accepts Android package names, TEESimulator's `pkg@user` form and `uid:N` tokens.
/// Anything else (spaces, shell metacharacters, section brackets) would corrupt a backend's
/// config file, so it is rejected up front.
pub(crate) fn is_valid_entry(entry: &str) -> bool {
    if let Some(uid) = entry.strip_prefix("uid:") {
        return !uid.is_empty() && uid.bytes().all(|byte| byte.is_ascii_digit());
    }

    let (package, user) = match entry.split_once('@') {
        Some((package, user)) => (package, Some(user)),
        None => (entry, None),
    };
    if user.is_some_and(|user| user.is_empty() || !user.bytes().all(|byte| byte.is_ascii_digit())) {
        return false;
    }

    !package.is_empty()
        && !package.starts_with('.')
        && !package.ends_with('.')
        && package
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'.')
}

fn clean(name: &str) -> String {
    name.trim().trim_end_matches(['!', '?']).trim().to_owned()
}

/// Trims, strips mode markers, drops invalid names, de-duplicates and sorts.
pub(crate) fn normalize_packages(values: Vec<String>) -> Vec<String> {
    values
        .iter()
        .map(|value| clean(value))
        .filter(|value| is_valid_entry(value))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// De-duplicates targets (last mode wins), drops invalid names and sorts by package. When
/// the backend has no per-app modes every target is reset to [`TargetMode::Auto`].
pub(crate) fn normalize_targets(
    targets: Vec<TargetEntry>,
    supports_mode: bool,
) -> Vec<TargetEntry> {
    let mut by_package = BTreeMap::new();
    for entry in targets {
        let package_name = clean(&entry.package_name);
        if is_valid_entry(&package_name) {
            let mode = if supports_mode {
                entry.mode
            } else {
                TargetMode::Auto
            };
            by_package.insert(package_name, mode);
        }
    }

    by_package
        .into_iter()
        .map(|(package_name, mode)| TargetEntry { package_name, mode })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{is_valid_entry, normalize_packages, normalize_targets};
    use crate::model::{TargetEntry, TargetMode};

    #[test]
    fn validates_entry_shapes() {
        assert!(is_valid_entry("com.google.android.gms"));
        assert!(is_valid_entry("com.example.app@10"));
        assert!(is_valid_entry("uid:10123"));
        assert!(!is_valid_entry("bad name"));
        assert!(!is_valid_entry("[target]"));
        assert!(!is_valid_entry("com.a;rm -rf /"));
        assert!(!is_valid_entry("uid:"));
        assert!(!is_valid_entry("com.a@"));
    }

    #[test]
    fn normalize_packages_strips_markers_and_dedupes() {
        assert_eq!(
            normalize_packages(vec![
                " com.b! ".into(),
                "com.a".into(),
                "com.a?".into(),
                "".into()
            ]),
            vec!["com.a", "com.b"]
        );
    }

    #[test]
    fn normalize_targets_resets_modes_without_support() {
        let targets = vec![TargetEntry {
            package_name: "com.a!".into(),
            mode: TargetMode::Generate,
        }];

        assert_eq!(
            normalize_targets(targets.clone(), true)[0].mode,
            TargetMode::Generate
        );
        assert_eq!(normalize_targets(targets, false)[0].mode, TargetMode::Auto);
    }
}
