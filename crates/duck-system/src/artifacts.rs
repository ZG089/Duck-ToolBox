//! Files the tools wrote to the shared outputs directory.

use anyhow::{Context as _, Result};
use duck_core::{
    AppPaths,
    fs::{list_files_recursive, modified_unix},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ArtifactFile {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified_unix: u64,
}

#[derive(Debug, Serialize)]
pub struct ArtifactsData {
    pub outputs: Vec<ArtifactFile>,
    pub outputs_dir: String,
    pub profile_path: String,
    pub profile_secrets_path: String,
    pub log_path: String,
}

pub fn list(paths: &AppPaths) -> Result<ArtifactsData> {
    paths.ensure_runtime_dirs()?;
    let mut outputs = Vec::new();

    for path in list_files_recursive(&paths.outputs_dir)? {
        let metadata =
            std::fs::metadata(&path).with_context(|| format!("read {}", path.display()))?;
        outputs.push(ArtifactFile {
            name: path
                .strip_prefix(&paths.outputs_dir)
                .unwrap_or(&path)
                .display()
                .to_string(),
            path: path.display().to_string(),
            size: metadata.len(),
            modified_unix: modified_unix(&metadata),
        });
    }

    outputs.sort_by_key(|file| std::cmp::Reverse(file.modified_unix));

    Ok(ArtifactsData {
        outputs,
        outputs_dir: paths.outputs_dir.display().to_string(),
        profile_path: paths.profile_path.display().to_string(),
        profile_secrets_path: paths.profile_secrets_path.display().to_string(),
        log_path: paths.log_path.display().to_string(),
    })
}
