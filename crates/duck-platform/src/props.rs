//! System properties via `getprop` and the root manager's `resetprop`.

use std::collections::BTreeMap;

use anyhow::Result;

use crate::exec;

/// `resetprop` locations shipped by KernelSU, APatch and Magisk.
const RESETPROP_CANDIDATES: &[&str] = &[
    "/data/adb/ksu/bin/resetprop",
    "/data/adb/ap/bin/resetprop",
    "/data/adb/magisk/resetprop",
];

/// Reads a single property; missing properties read as an empty string, like `getprop`.
pub fn get(name: &str) -> String {
    exec::stdout_or_empty("getprop", &[name]).trim().to_owned()
}

/// Returns the first non-empty value among `names`.
pub fn first(names: &[&str]) -> String {
    names
        .iter()
        .map(|name| get(name))
        .find(|value| !value.is_empty())
        .unwrap_or_default()
}

/// Reads every property in one `getprop` call.
pub fn all() -> BTreeMap<String, String> {
    parse_listing(&exec::stdout_or_empty("getprop", &[]))
}

/// Sets a property without triggering property-service side effects (`resetprop -n`),
/// which is what KernelSU recommends for boot scripts.
pub fn reset(name: &str, value: &str) -> Result<()> {
    exec::stdout(&resetprop(), &["-n", name, value]).map(|_| ())
}

pub fn resetprop() -> String {
    exec::find_tool("resetprop", RESETPROP_CANDIDATES)
}

/// Parses `getprop` output lines of the form `[name]: [value]`.
pub fn parse_listing(output: &str) -> BTreeMap<String, String> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let (name, value) = line.strip_prefix('[')?.split_once("]: [")?;
            Some((name.to_owned(), value.strip_suffix(']')?.to_owned()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_listing;

    #[test]
    fn parses_getprop_listing() {
        let props = parse_listing(
            "[ro.product.model]: [Pixel 9]\n[ro.boot.vbmeta.digest]: []\ngarbage\n[a.b]: [x]: [y]]\n",
        );

        assert_eq!(props.get("ro.product.model").unwrap(), "Pixel 9");
        assert_eq!(props.get("ro.boot.vbmeta.digest").unwrap(), "");
        assert_eq!(props.get("a.b").unwrap(), "x]: [y]");
        assert_eq!(props.len(), 3);
    }
}
