//! `duckd`: every feature crate registered in [`FEATURES`] becomes `duckd <id> ...`, and every
//! invocation prints exactly one JSON envelope on stdout.

use clap::{Arg, ArgAction, error::ErrorKind};
use duck_core::{
    Context, Feature, FeatureInfo, Sysroot,
    command::{CommandFailure, CommandOutput, CommandResult},
    envelope,
};
use serde::Serialize;

static FEATURES: &[&dyn Feature] = &[
    #[cfg(feature = "rkp")]
    &duck_rkp::FEATURE,
    #[cfg(feature = "device-ids")]
    &duck_device_ids::FEATURE,
    #[cfg(feature = "tricky-store")]
    &duck_tricky_store::FEATURE,
    #[cfg(feature = "system")]
    &duck_system::FEATURE,
];

const MANIFEST_COMMAND: &str = "features";

#[derive(Debug, Serialize)]
struct Manifest {
    binary_version: &'static str,
    api: u32,
    features: Vec<FeatureInfo>,
}

fn cli() -> clap::Command {
    clap::Command::new("duckd")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Duck ToolBox backend. Every subcommand prints exactly one JSON envelope on stdout.")
        .arg(
            Arg::new("json")
                .long("json")
                .global(true)
                .action(ArgAction::SetTrue)
                .help("Accepted for compatibility; output is always JSON"),
        )
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            clap::Command::new(MANIFEST_COMMAND)
                .about("List the features compiled into this binary"),
        )
        .subcommands(FEATURES.iter().map(|feature| feature.command()))
}

#[tokio::main]
async fn main() {
    let matches = match cli().try_get_matches() {
        Ok(matches) => matches,
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            error.exit()
        }
        Err(error) => {
            let failure = CommandFailure::new(
                "bootstrap",
                "usage_error",
                &anyhow::anyhow!(error.render().to_string()),
                None,
            );
            envelope::emit(&envelope::failure(&failure));
            std::process::exit(2);
        }
    };

    let paths = match duck_core::AppPaths::discover() {
        Ok(paths) => paths,
        Err(error) => {
            let failure = CommandFailure::new("bootstrap", "bootstrap_error", &error, None);
            envelope::emit(&envelope::failure(&failure));
            std::process::exit(1);
        }
    };
    let ctx = Context {
        paths,
        sysroot: Sysroot::from_env(),
    };

    let result = match matches.subcommand() {
        Some((MANIFEST_COMMAND, _)) => manifest(),
        Some((id, sub)) => match FEATURES.iter().find(|feature| feature.info().id == id) {
            Some(feature) => feature.run(sub, &ctx).await,
            None => unreachable!("clap only accepts registered subcommands"),
        },
        None => unreachable!("a subcommand is required"),
    };
    emit(&ctx, result);
}

fn manifest() -> CommandResult {
    CommandOutput::new(
        "features",
        Manifest {
            binary_version: env!("CARGO_PKG_VERSION"),
            api: envelope::API_VERSION,
            features: FEATURES.iter().map(|feature| feature.info()).collect(),
        },
    )
}

fn emit(ctx: &Context, result: CommandResult) {
    let (value, failed) = match result {
        Ok(output) => (envelope::success(&output), false),
        Err(failure) => (envelope::failure(&failure), true),
    };

    duck_core::log::append(&ctx.paths, &value);
    envelope::emit(&value);

    if failed {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{FEATURES, cli};

    #[test]
    fn cli_is_consistent() {
        cli().debug_assert();
    }

    #[test]
    fn feature_ids_are_unique() {
        let mut ids: Vec<_> = FEATURES.iter().map(|feature| feature.info().id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), FEATURES.len());
    }
}
