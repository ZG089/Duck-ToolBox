//! OhMyKeymint's `injector.toml` (`injector/src/config.rs`).
//!
//! It is TOML, except that per-package settings are written as `[scoop.<package>]` tables
//! next to the `scoop` array, which plain TOML rejects as a redefinition. OhMyKeymint
//! rewrites those headers to `[scoop_details."<package>"]` before parsing (its
//! `preprocess_config`) and writes them back as `[scoop.<package>]`; so does this module.
//!
//! The file keeps exactly the keys it has: the released OhMyKeymint (v1.2.0) refuses any key
//! it does not know, `version` included, while newer builds add `version` on their own.

use anyhow::{Context, Result, bail};
use toml_edit::DocumentMut;

const SCOOP_HEADER: &str = "[scoop.";
const DETAILS_HEADER: &str = "[scoop_details.";

pub(super) fn parse(raw: &str) -> Result<DocumentMut> {
    let raw = raw.strip_prefix('\u{feff}').unwrap_or(raw);
    let mut standard = String::with_capacity(raw.len());
    for (index, line) in raw.split_inclusive('\n').enumerate() {
        standard.push_str(&scoop_to_details(line, index + 1)?);
    }
    standard
        .parse::<DocumentMut>()
        .context("parse OhMyKeymint injector.toml")
}

pub(super) fn render(document: &DocumentMut) -> String {
    document
        .to_string()
        .split_inclusive('\n')
        .map(details_to_scoop)
        .collect()
}

/// `[scoop.<package>]` to `[scoop_details."<package>"]`; other lines pass unchanged.
fn scoop_to_details(line: &str, line_no: usize) -> Result<String> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with(SCOOP_HEADER) {
        return Ok(line.to_owned());
    }
    let Some(close) = trimmed.find(']') else {
        bail!("injector.toml line {line_no}: unterminated [scoop.<package>] header");
    };
    let fragment = trimmed[SCOOP_HEADER.len()..close].trim();
    let package = if fragment.starts_with(['"', '\'']) {
        let wrapped: DocumentMut = format!("package = {fragment}")
            .parse()
            .with_context(|| format!("injector.toml line {line_no}: bad package name"))?;
        wrapped["package"].as_str().unwrap_or_default().to_owned()
    } else {
        fragment.to_owned()
    };
    if package.trim().is_empty() {
        bail!("injector.toml line {line_no}: empty scoop package name");
    }
    let leading = &line[..line.len() - trimmed.len()];
    Ok(format!(
        "{leading}{DETAILS_HEADER}{}]{}",
        toml_edit::Value::from(package.trim()),
        &trimmed[close + 1..]
    ))
}

/// The reverse of [`scoop_to_details`], in OhMyKeymint's `[scoop.<package>]` form.
fn details_to_scoop(line: &str) -> String {
    let trimmed = line.trim_start();
    let Some(rest) = trimmed.strip_prefix(DETAILS_HEADER) else {
        return line.to_owned();
    };
    let Some(close) = rest.rfind(']') else {
        return line.to_owned();
    };
    let key = &rest[..close];
    let package = format!("package = {key}")
        .parse::<DocumentMut>()
        .ok()
        .and_then(|wrapped| wrapped["package"].as_str().map(str::to_owned))
        .unwrap_or_else(|| key.to_owned());
    let leading = &line[..line.len() - trimmed.len()];
    let bare = package
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'));
    let name = if bare {
        package
    } else {
        toml_edit::Value::from(package).to_string()
    };
    format!("{leading}{SCOOP_HEADER}{name}]{}", &rest[close + 1..])
}

#[cfg(test)]
mod tests {
    use super::{parse, render};

    const WITH_DETAILS: &str = "# Only packages listed in `scoop` are intercepted.\n\
        scoop = [\"io.github.vvb2060.keyattestation\"]\n\n\
        [main]\nenabled = true\n\n\
        [scoop.io.github.vvb2060.keyattestation]\nmode = \"strict\"\n\n\
        [scoop.\"com.example.quoted\"]\n";

    #[test]
    fn reads_per_package_tables_next_to_the_scoop_array() {
        let document = parse(WITH_DETAILS).unwrap();
        assert_eq!(
            document["scoop_details"]["io.github.vvb2060.keyattestation"]["mode"].as_str(),
            Some("strict")
        );
        assert!(
            document["scoop_details"]
                .get("com.example.quoted")
                .is_some()
        );
    }

    #[test]
    fn writes_them_back_in_ohmykeymint_form() {
        let mut document = parse(WITH_DETAILS).unwrap();
        document["scoop"] = toml_edit::value(toml_edit::Array::from_iter(["com.example.bank"]));
        let rendered = render(&document);
        assert!(rendered.contains("[scoop.io.github.vvb2060.keyattestation]\nmode = \"strict\""));
        assert!(rendered.contains("[scoop.com.example.quoted]"));
        assert!(!rendered.contains("scoop_details"));
        assert!(rendered.starts_with("# Only packages"));
        assert!(!rendered.contains("version"));
    }

    #[test]
    fn rejects_a_broken_header() {
        assert!(parse("[scoop.unterminated\n").is_err());
        assert!(parse("[scoop.]\n").is_err());
    }
}
