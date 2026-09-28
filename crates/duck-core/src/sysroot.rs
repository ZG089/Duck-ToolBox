use std::{
    env,
    path::{Component, Path, PathBuf},
};

/// Maps absolute Android paths (`/data/adb/...`) onto the filesystem the backend runs on.
///
/// On a device the root is `/`. Integration tests point `DUCK_TOOLBOX_SYSROOT` at a
/// temporary directory so every feature can be exercised without a phone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sysroot {
    root: PathBuf,
}

impl Sysroot {
    pub const ENV: &'static str = "DUCK_TOOLBOX_SYSROOT";

    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn from_env() -> Self {
        match env::var_os(Self::ENV) {
            Some(root) if !root.is_empty() => Self::new(root),
            _ => Self::new("/"),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Resolves an absolute device path inside this sysroot. Relative paths and `..`
    /// components are rejected by dropping them, so callers can't escape the root.
    pub fn path(&self, device_path: impl AsRef<Path>) -> PathBuf {
        let mut resolved = self.root.clone();
        for component in device_path.as_ref().components() {
            if let Component::Normal(part) = component {
                resolved.push(part);
            }
        }
        resolved
    }

    /// Converts a host path back to the device path it represents, for display.
    pub fn display_path(&self, host_path: &Path) -> String {
        match host_path.strip_prefix(&self.root) {
            Ok(relative) if self.root != Path::new("/") => {
                format!("/{}", relative.display())
            }
            _ => host_path.display().to_string(),
        }
    }
}

impl Default for Sysroot {
    fn default() -> Self {
        Self::new("/")
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::Sysroot;

    #[test]
    fn device_root_is_identity() {
        let sysroot = Sysroot::default();
        assert_eq!(
            sysroot.path("/data/adb/modules"),
            PathBuf::from("/data/adb/modules")
        );
    }

    #[test]
    fn test_root_prefixes_device_paths() {
        let sysroot = Sysroot::new("/tmp/fake");
        let host = sysroot.path("/data/adb/tricky_store/target.txt");

        assert_eq!(
            host,
            PathBuf::from("/tmp/fake/data/adb/tricky_store/target.txt")
        );
        assert_eq!(
            sysroot.display_path(&host),
            "/data/adb/tricky_store/target.txt"
        );
    }

    #[test]
    fn parent_components_cannot_escape() {
        let sysroot = Sysroot::new("/tmp/fake");
        assert_eq!(
            sysroot.path("/data/../../etc/passwd"),
            Path::new("/tmp/fake/data/etc/passwd")
        );
    }
}
