//! Commands that need the whole feature registry rather than one feature.

use anyhow::{Context as _, Result};
use duck_core::{
    Context, Feature, FeatureInfo, IntoCommandResult,
    command::{CommandOutput, CommandResult},
    envelope, internal_error,
};
use duck_platform::{modules::parse_module_prop, root};
use serde::Serialize;

pub const MANIFEST: &str = "features";
pub const DESCRIBE: &str = "describe";

#[derive(Debug, Serialize)]
struct Manifest {
    binary_version: &'static str,
    api: u32,
    features: Vec<FeatureInfo>,
}

#[derive(Debug, Serialize)]
struct Description {
    description: String,
    /// Whether the root manager accepted the override (KernelSU only).
    applied: bool,
}

pub fn subcommands() -> [clap::Command; 2] {
    [
        clap::Command::new(MANIFEST).about("List the features compiled into this binary"),
        clap::Command::new(DESCRIBE).about(
            "Show every feature's status in the root manager's module list (KernelSU \
             override.description)",
        ),
    ]
}

pub fn manifest(features: &[&dyn Feature]) -> CommandResult {
    CommandOutput::new(
        MANIFEST,
        Manifest {
            binary_version: env!("CARGO_PKG_VERSION"),
            api: envelope::API_VERSION,
            features: features.iter().map(|feature| feature.info()).collect(),
        },
    )
}

pub fn describe(features: &[&dyn Feature], ctx: &Context) -> CommandResult {
    describe_inner(features, ctx).into_command(DESCRIBE, internal_error)
}

fn describe_inner(features: &[&dyn Feature], ctx: &Context) -> Result<Description> {
    let prop_path = ctx.paths.root.join("module.prop");
    let props = parse_module_prop(
        &std::fs::read_to_string(&prop_path)
            .with_context(|| format!("read {}", prop_path.display()))?,
    );
    let id = props.get("id").cloned().unwrap_or_default();
    let base = props.get("description").cloned().unwrap_or_default();

    let lines: Vec<String> = features
        .iter()
        .filter_map(|feature| feature.status_line(ctx))
        .collect();
    let description = if lines.is_empty() {
        base
    } else {
        format!("{} | {base}", lines.join("; "))
    };

    let applied = match root::detect(&ctx.sysroot) {
        Some(manager) if manager.kind == root::RootManagerKind::KernelSu => {
            manager.set_module_description(&id, &description)?;
            true
        }
        _ => false,
    };
    Ok(Description {
        description,
        applied,
    })
}
