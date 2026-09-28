use anyhow::{Context as _, Result};
use clap::Subcommand;
use duck_core::{
    AppPaths, CommandResult, Context, IntoCommandResult, fs::list_files_recursive,
    fs::modified_unix, internal_error,
};
use serde::Serialize;

#[derive(Debug, Subcommand)]
pub enum ArtifactCommand {
    /// List generated output files and the runtime paths.
    List,
}

#[derive(Debug, Serialize)]
struct ArtifactFile {
    name: String,
    path: String,
    size: u64,
    modified_unix: u64,
}

#[derive(Debug, Serialize)]
struct ArtifactsData {
    outputs: Vec<ArtifactFile>,
    profile_path: String,
    profile_secrets_path: String,
    log_path: String,
}

pub fn run(command: ArtifactCommand, ctx: &Context) -> CommandResult {
    match command {
        ArtifactCommand::List => list(&ctx.paths).into_command("artifacts.list", internal_error),
    }
}

fn list(paths: &AppPaths) -> Result<ArtifactsData> {
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
        profile_path: paths.profile_path.display().to_string(),
        profile_secrets_path: paths.profile_secrets_path.display().to_string(),
        log_path: paths.log_path.display().to_string(),
    })
}
