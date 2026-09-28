//! The "unnecessary apps" list: root managers, root tools and general apps that never
//! check bootloader state. Tricky Addon maintains it upstream; a copy ships in the binary
//! so the feature works offline, and a refresh fetches the latest list.

use anyhow::{Context as _, Result};
use duck_core::{Context, fs::write_string_atomic};
use serde::{Deserialize, Serialize};

use crate::error::TrickyError;

const BUNDLED: &str = include_str!("../assets/more-exclude.json");
const CACHE_FILE: &str = "tricky-store-exclude.json";
const REMOTE_URL: &str = "https://raw.githubusercontent.com/KOWX712/Tricky-Addon-Update-Target-List/main/more-exclude.json";
/// Tricky Addon's fallback proxy for regions where raw.githubusercontent.com is blocked.
const MIRROR_PREFIX: &str = "https://gh.sevencdn.com/";

#[derive(Debug, Deserialize)]
struct ExcludeFile {
    data: Vec<Category>,
}

#[derive(Debug, Deserialize)]
struct Category {
    apps: Vec<App>,
}

#[derive(Debug, Deserialize)]
struct App {
    #[serde(rename = "package-name")]
    package_name: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ListSource {
    Bundled,
    Cache,
    Remote,
}

pub fn parse(raw: &str) -> Result<Vec<String>> {
    let file: ExcludeFile = serde_json::from_str(raw).context("parse unnecessary-apps list")?;
    Ok(crate::targets::normalize_packages(
        file.data
            .into_iter()
            .flat_map(|category| category.apps)
            .map(|app| app.package_name)
            .collect(),
    ))
}

/// Returns the list and where it came from. With `refresh`, the remote list replaces the
/// cache; a failed refresh falls back to the cached or bundled copy instead of erroring.
pub async fn load(ctx: &Context, refresh: bool) -> Result<(Vec<String>, ListSource)> {
    let cache_path = ctx.paths.var_dir.join(CACHE_FILE);

    if refresh && let Ok(raw) = fetch_remote().await {
        let list = parse(&raw)?;
        ctx.paths.ensure_runtime_dirs()?;
        write_string_atomic(&cache_path, &raw)?;
        return Ok((list, ListSource::Remote));
    }

    if let Some(raw) = duck_core::fs::read_optional(&cache_path)?
        && let Ok(list) = parse(&raw)
    {
        return Ok((list, ListSource::Cache));
    }

    Ok((parse(BUNDLED)?, ListSource::Bundled))
}

async fn fetch_remote() -> Result<String> {
    let client = duck_platform::net::http_client()?;
    for url in [
        REMOTE_URL.to_owned(),
        format!("{MIRROR_PREFIX}{REMOTE_URL}"),
    ] {
        if let Ok(response) = client.get(&url).send().await
            && response.status().is_success()
            && let Ok(body) = response.text().await
        {
            return Ok(body);
        }
    }
    Err(TrickyError::Download("could not reach the unnecessary-apps list".into()).into())
}

/// Magisk's DenyList; empty on managers without one.
pub fn denylist(ctx: &Context) -> Vec<String> {
    duck_platform::root::detect(&ctx.sysroot)
        .map(|manager| manager.denylist())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{BUNDLED, parse};

    #[test]
    fn bundled_list_contains_root_managers() {
        let list = parse(BUNDLED).unwrap();
        assert!(list.iter().any(|app| app == "me.weishu.kernelsu"));
        assert!(list.iter().any(|app| app == "com.topjohnwu.magisk"));
    }
}
