//! Installed packages via the framework's `pm` shell command.

use anyhow::Result;

use crate::exec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageFilter {
    All,
    /// Third-party apps (`pm list packages -3`).
    User,
    /// System apps (`pm list packages -s`).
    System,
}

impl PackageFilter {
    fn flag(self) -> Option<&'static str> {
        match self {
            Self::All => None,
            Self::User => Some("-3"),
            Self::System => Some("-s"),
        }
    }
}

/// Lists package names. Returns an empty list when `pm` is unavailable, e.g. early in boot.
pub fn list(filter: PackageFilter) -> Vec<String> {
    let mut args = vec!["list", "packages"];
    args.extend(filter.flag());
    parse_list(&exec::stdout_or_empty("pm", &args))
}

/// Lists `(package, base APK path)` pairs in a single `pm list packages -f` call.
pub fn list_with_paths(filter: PackageFilter) -> Result<Vec<(String, String)>> {
    let mut args = vec!["list", "packages", "-f"];
    args.extend(filter.flag());
    Ok(parse_list_with_paths(&exec::stdout("pm", &args)?))
}

pub fn parse_list(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| line.trim().strip_prefix("package:"))
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Parses `package:/data/app/~~x==/com.foo-y==/base.apk=com.foo`. The package name
/// follows the last `=`, since APK paths may themselves contain `=`.
pub fn parse_list_with_paths(output: &str) -> Vec<(String, String)> {
    output
        .lines()
        .filter_map(|line| line.trim().strip_prefix("package:"))
        .filter_map(|entry| entry.rsplit_once('='))
        .filter(|(path, name)| !path.is_empty() && !name.is_empty())
        .map(|(path, name)| (name.to_owned(), path.to_owned()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{parse_list, parse_list_with_paths};

    #[test]
    fn parses_package_names() {
        assert_eq!(
            parse_list("package:com.a\npackage: com.b \nnoise\npackage:\n"),
            vec!["com.a", "com.b"]
        );
    }

    #[test]
    fn parses_package_paths_with_equals_in_path() {
        let parsed = parse_list_with_paths(
            "package:/data/app/~~AbC==/com.foo-XyZ==/base.apk=com.foo\npackage:/system/app/Bar/Bar.apk=com.bar\n",
        );

        assert_eq!(
            parsed,
            vec![
                (
                    "com.foo".to_owned(),
                    "/data/app/~~AbC==/com.foo-XyZ==/base.apk".to_owned()
                ),
                ("com.bar".to_owned(), "/system/app/Bar/Bar.apk".to_owned()),
            ]
        );
    }
}
