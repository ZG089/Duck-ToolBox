use std::io::{self, Read};

use anyhow::{Context, Result, bail};
use duck_core::AppPaths;
use serde_json::{Value, json};

use crate::{
    cli::{ProfileArgs, ProfileSaveArgs},
    profile::{ProfileData, clear_profile, save_profile, show_profile},
};

pub(crate) fn show(paths: &AppPaths, args: &ProfileArgs) -> Result<Value> {
    let profile = show_profile(paths, args.profile.as_deref())?;
    Ok(json!({ "profile": profile, "paths": paths }))
}

pub(crate) fn save(paths: &AppPaths, args: &ProfileSaveArgs) -> Result<Value> {
    if !args.stdin_json {
        bail!("profile save requires `--stdin-json`");
    }

    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .context("read profile JSON from stdin")?;
    let profile = serde_json::from_str::<ProfileData>(&input).context("parse profile JSON")?;

    let profile = save_profile(paths, args.profile.as_deref(), &profile)?;
    Ok(json!({ "profile": profile, "paths": paths }))
}

pub(crate) fn clear(paths: &AppPaths, args: &ProfileArgs) -> Result<Value> {
    clear_profile(paths, args.profile.as_deref())?;
    Ok(json!({ "cleared": true, "paths": paths }))
}
