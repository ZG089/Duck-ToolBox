use std::{
    fs::{self, File},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, anyhow, bail};

/// Reads a UTF-8 file, returning `None` when it does not exist.
pub fn read_optional(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(Some(content)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("read {}", path.display())),
    }
}

pub fn write_string_atomic(path: &Path, contents: &str) -> Result<()> {
    write_bytes_atomic(path, contents.as_bytes())
}

/// Writes through a temporary sibling file and a rename so readers never observe a
/// half-written file.
pub fn write_bytes_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    write_atomic_with(path, bytes, |_| Ok(()))
}

/// Like [`write_bytes_atomic`], but the replacement keeps the permission bits and owner of
/// the file it replaces. Other modules' configs (Tricky Store, TEESimulator, OhMyKeymint)
/// are read by daemons running under their own uid, so resetting ownership would break them.
pub fn write_bytes_preserving(path: &Path, bytes: &[u8]) -> Result<()> {
    let original = fs::metadata(path).ok();
    write_atomic_with(path, bytes, |temp| {
        if let Some(metadata) = &original {
            copy_ownership(metadata, temp)?;
        }
        Ok(())
    })
}

/// Copies `path` to `path.bak`, replacing an older backup. Returns the backup path, or
/// `None` when there was nothing to back up.
pub fn backup_copy(path: &Path) -> Result<Option<PathBuf>> {
    if !path.is_file() {
        return Ok(None);
    }

    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow!("{} has no file name", path.display()))?;
    let backup = path.with_file_name(format!("{}.bak", file_name.to_string_lossy()));
    fs::copy(path, &backup)
        .with_context(|| format!("back up {} -> {}", path.display(), backup.display()))?;
    Ok(Some(backup))
}

pub fn create_unique_dir(parent: &Path, prefix: &str) -> Result<PathBuf> {
    fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;

    for attempt in 0..32_u32 {
        let path = parent.join(format!("{prefix}-{}", unique_suffix(attempt)));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error).with_context(|| format!("create {}", path.display())),
        }
    }

    bail!(
        "failed to create a unique directory under {}",
        parent.display()
    )
}

pub fn list_files_recursive(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry
                .metadata()
                .with_context(|| format!("read metadata for {}", path.display()))?;

            if metadata.is_dir() {
                stack.push(path);
            } else if metadata.is_file() {
                files.push(path);
            }
        }
    }

    Ok(files)
}

/// Seconds since the Unix epoch of the file's last modification, or `0`.
pub fn modified_unix(metadata: &fs::Metadata) -> u64 {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn write_atomic_with(
    path: &Path,
    bytes: &[u8],
    prepare: impl FnOnce(&Path) -> Result<()>,
) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("{} has no parent directory", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;

    let temp_path = unique_temp_path(parent, path);
    let result = (|| -> Result<()> {
        let mut file =
            File::create(&temp_path).with_context(|| format!("create {}", temp_path.display()))?;
        file.write_all(bytes)
            .with_context(|| format!("write {}", temp_path.display()))?;
        file.sync_all()
            .with_context(|| format!("sync {}", temp_path.display()))?;
        drop(file);

        prepare(&temp_path)?;
        replace_file(&temp_path, path)
    })();

    if result.is_err() && temp_path.exists() {
        let _ = fs::remove_file(&temp_path);
    }

    result
}

#[cfg(unix)]
fn copy_ownership(original: &fs::Metadata, target: &Path) -> Result<()> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    fs::set_permissions(target, fs::Permissions::from_mode(original.mode() & 0o7777))
        .with_context(|| format!("chmod {}", target.display()))?;
    // Only root can hand files to another uid; elsewhere keeping the mode is enough.
    let _ = std::os::unix::fs::chown(target, Some(original.uid()), Some(original.gid()));
    Ok(())
}

#[cfg(not(unix))]
fn copy_ownership(_original: &fs::Metadata, _target: &Path) -> Result<()> {
    Ok(())
}

fn replace_file(temp_path: &Path, destination: &Path) -> Result<()> {
    match fs::rename(temp_path, destination) {
        Ok(()) => Ok(()),
        Err(_) if destination.exists() => {
            fs::remove_file(destination)
                .with_context(|| format!("remove {}", destination.display()))?;
            fs::rename(temp_path, destination).with_context(|| {
                format!(
                    "rename {} -> {}",
                    temp_path.display(),
                    destination.display()
                )
            })
        }
        Err(error) => Err(error).with_context(|| {
            format!(
                "rename {} -> {}",
                temp_path.display(),
                destination.display()
            )
        }),
    }
}

fn unique_temp_path(parent: &Path, destination: &Path) -> PathBuf {
    let stem = destination
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("duck-toolbox");
    parent.join(format!(".{stem}.tmp-{}", unique_suffix(0)))
}

fn unique_suffix(attempt: u32) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}-{}-{attempt}", process::id())
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use super::{
        backup_copy, create_unique_dir, list_files_recursive, read_optional, write_bytes_atomic,
        write_bytes_preserving,
    };

    fn temp_root(name: &str) -> std::path::PathBuf {
        let root = env::temp_dir().join(format!("duck-core-fs-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn write_bytes_atomic_overwrites_existing_file() {
        let root = temp_root("atomic");
        let file = root.join("value.txt");

        write_bytes_atomic(&file, b"first").unwrap();
        write_bytes_atomic(&file, b"second").unwrap();

        assert_eq!(fs::read(&file).unwrap(), b"second");
    }

    #[cfg(unix)]
    #[test]
    fn write_bytes_preserving_keeps_mode() {
        use std::os::unix::fs::PermissionsExt;

        let root = temp_root("preserve");
        let file = root.join("config.toml");
        fs::write(&file, b"old").unwrap();
        fs::set_permissions(&file, fs::Permissions::from_mode(0o640)).unwrap();

        write_bytes_preserving(&file, b"new").unwrap();

        assert_eq!(fs::read(&file).unwrap(), b"new");
        assert_eq!(
            fs::metadata(&file).unwrap().permissions().mode() & 0o777,
            0o640
        );
    }

    #[test]
    fn read_optional_distinguishes_missing_files() {
        let root = temp_root("optional");
        assert!(read_optional(&root.join("missing")).unwrap().is_none());

        fs::write(root.join("present"), "hi").unwrap();
        assert_eq!(
            read_optional(&root.join("present")).unwrap().as_deref(),
            Some("hi")
        );
    }

    #[test]
    fn backup_copy_keeps_the_original() {
        let root = temp_root("backup");
        let file = root.join("keybox.xml");
        assert!(backup_copy(&file).unwrap().is_none());

        fs::write(&file, "original").unwrap();
        let backup = backup_copy(&file).unwrap().unwrap();

        assert_eq!(backup.file_name().unwrap(), "keybox.xml.bak");
        assert_eq!(fs::read_to_string(&backup).unwrap(), "original");
        assert_eq!(fs::read_to_string(&file).unwrap(), "original");
    }

    #[test]
    fn create_unique_dir_creates_distinct_directories() {
        let root = temp_root("dirs");
        let first = create_unique_dir(&root, "run").unwrap();
        let second = create_unique_dir(&root, "run").unwrap();

        assert_ne!(first, second);
        assert!(first.is_dir());
        assert!(second.is_dir());
    }

    #[test]
    fn list_files_recursive_returns_nested_files_only() {
        let root = temp_root("files");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("a.txt"), b"a").unwrap();
        fs::write(nested.join("b.txt"), b"b").unwrap();

        let mut files = list_files_recursive(&root)
            .unwrap()
            .into_iter()
            .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        files.sort();

        assert_eq!(files, vec!["a.txt", "b.txt"]);
    }
}
