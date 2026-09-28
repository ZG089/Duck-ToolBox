//! Directory listing for the WebUI file picker, shared by every tool that imports files.

use anyhow::{Context as _, Result};
use duck_core::{Context, fs::modified_unix};
use serde::Serialize;

const DEFAULT_DIR: &str = "/storage/emulated/0/Download";

#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub directory: bool,
    pub size: u64,
    pub modified_unix: u64,
}

#[derive(Debug, Serialize)]
pub struct FileListData {
    pub path: String,
    pub parent: Option<String>,
    pub entries: Vec<FileEntry>,
}

/// Lists directories and files matching `extension` (empty lists all files). Directories
/// come first, then a case-insensitive name sort.
pub fn list(ctx: &Context, directory: &str, extension: &str) -> Result<FileListData> {
    let device_dir = if directory.trim().is_empty() {
        DEFAULT_DIR.to_owned()
    } else {
        directory.trim().to_owned()
    };
    let host_dir = ctx.sysroot.path(&device_dir);
    let wanted = extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase();

    let mut entries = Vec::new();
    for entry in
        std::fs::read_dir(&host_dir).with_context(|| format!("read directory {device_dir}"))?
    {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let matches = wanted.is_empty()
            || entry
                .path()
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case(&wanted));

        if !metadata.is_dir() && !matches {
            continue;
        }

        entries.push(FileEntry {
            path: format!("{}/{name}", device_dir.trim_end_matches('/')),
            directory: metadata.is_dir(),
            size: if metadata.is_file() {
                metadata.len()
            } else {
                0
            },
            modified_unix: modified_unix(&metadata),
            name,
        });
    }

    entries.sort_by(|left, right| {
        right
            .directory
            .cmp(&left.directory)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });

    let parent = host_dir
        .parent()
        .filter(|parent| parent.starts_with(ctx.sysroot.root()))
        .map(|parent| ctx.sysroot.display_path(parent));

    Ok(FileListData {
        path: device_dir,
        parent,
        entries,
    })
}
