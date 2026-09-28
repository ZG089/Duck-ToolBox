//! Custom keybox providers: named URLs with an optional decode pipeline.
//!
//! Import/export uses Tricky Addon's JSON format, so provider lists move between the two.

use anyhow::{Context as _, Result};
use duck_core::{Context, fs::write_string_atomic};
use serde::{Deserialize, Serialize};

use crate::{
    error::TrickyError,
    keybox::decode,
    state::{KeyboxProvider, State, default_providers},
};

const EXPORT_METADATA: &str = "tricky_addon_custom_keybox_config";
const DEFAULT_EXPORT_DIR: &str = "/storage/emulated/0/Download";

#[derive(Debug, Serialize, Deserialize)]
struct ExportFile {
    metadata: String,
    version: u32,
    entries: Vec<ExportEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportEntry {
    name: String,
    link: String,
    #[serde(default)]
    script: String,
}

pub fn list(ctx: &Context) -> Result<Vec<KeyboxProvider>> {
    Ok(State::load(&ctx.paths, &ctx.sysroot)?.providers)
}

pub fn save(ctx: &Context, providers: Vec<KeyboxProvider>) -> Result<Vec<KeyboxProvider>> {
    let providers = validated(providers)?;
    let mut state = State::load(&ctx.paths, &ctx.sysroot)?;
    state.providers = providers;
    state.save(&ctx.paths)?;
    Ok(state.providers)
}

pub fn reset(ctx: &Context) -> Result<Vec<KeyboxProvider>> {
    save(ctx, default_providers())
}

/// Merges providers from an exported file; entries with the same name are replaced.
pub fn import(ctx: &Context, device_path: &str) -> Result<Vec<KeyboxProvider>> {
    let host = ctx.sysroot.path(device_path);
    let raw = std::fs::read_to_string(&host).with_context(|| format!("read {device_path}"))?;
    import_content(ctx, &raw)
}

pub fn import_content(ctx: &Context, raw: &str) -> Result<Vec<KeyboxProvider>> {
    let file: ExportFile = serde_json::from_str(raw).context("parse provider export")?;
    if file.metadata != EXPORT_METADATA {
        return Err(TrickyError::BackendRule("not a keybox provider export file".into()).into());
    }

    let mut merged = list(ctx)?;
    for entry in file.entries {
        merged.retain(|existing| existing.name != entry.name);
        merged.push(KeyboxProvider {
            name: entry.name,
            url: entry.link,
            decode: entry.script,
        });
    }
    save(ctx, merged)
}

/// Writes the providers to `device_path`, or to the Download folder when omitted.
pub fn export(ctx: &Context, device_path: Option<&str>) -> Result<String> {
    let file = ExportFile {
        metadata: EXPORT_METADATA.into(),
        version: 1,
        entries: list(ctx)?
            .into_iter()
            .map(|provider| ExportEntry {
                name: provider.name,
                link: provider.url,
                script: provider.decode,
            })
            .collect(),
    };

    let target = match device_path {
        Some(path) => path.to_owned(),
        None => {
            let date = time::OffsetDateTime::now_utc().date();
            format!(
                "{DEFAULT_EXPORT_DIR}/duck-keybox-providers-{:04}{:02}{:02}.json",
                date.year(),
                u8::from(date.month()),
                date.day()
            )
        }
    };
    let body = serde_json::to_string_pretty(&file).context("serialize provider export")?;
    write_string_atomic(&ctx.sysroot.path(&target), &body)?;
    Ok(target)
}

fn validated(providers: Vec<KeyboxProvider>) -> Result<Vec<KeyboxProvider>> {
    let mut clean: Vec<KeyboxProvider> = Vec::new();
    for provider in providers {
        let name = provider.name.trim().to_owned();
        let url = provider.url.trim().to_owned();
        let decode_steps = provider.decode.trim().to_owned();
        if name.is_empty() {
            return Err(TrickyError::BackendRule("provider name must not be empty".into()).into());
        }
        if !(url.starts_with("https://") || url.starts_with("http://")) {
            return Err(TrickyError::BackendRule(format!(
                "provider `{name}` needs an http(s) URL"
            ))
            .into());
        }
        decode::check(&decode_steps)?;
        clean.retain(|existing| existing.name != name);
        clean.push(KeyboxProvider {
            name,
            url,
            decode: decode_steps,
        });
    }
    Ok(clean)
}

#[cfg(test)]
mod tests {
    use super::validated;
    use crate::state::KeyboxProvider;

    fn provider(name: &str, url: &str, decode: &str) -> KeyboxProvider {
        KeyboxProvider {
            name: name.into(),
            url: url.into(),
            decode: decode.into(),
        }
    }

    #[test]
    fn rejects_unsafe_or_incomplete_providers() {
        assert!(validated(vec![provider("", "https://x", "")]).is_err());
        assert!(validated(vec![provider("a", "file:///etc", "")]).is_err());
        assert!(validated(vec![provider("a", "https://x", "sh")]).is_err());
    }

    #[test]
    fn deduplicates_by_name() {
        let clean = validated(vec![
            provider("a", "https://one", ""),
            provider(" a ", "https://two", "base64 -d"),
        ])
        .unwrap();
        assert_eq!(clean.len(), 1);
        assert_eq!(clean[0].url, "https://two");
    }
}
