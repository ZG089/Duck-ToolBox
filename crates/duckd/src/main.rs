mod artifacts;
mod cli;

use clap::Parser;
use duck_core::{
    Context, Sysroot,
    command::{CommandFailure, CommandResult},
    envelope,
};

use cli::{Cli, Feature};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

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

    let result = dispatch(cli.command, &ctx).await;
    emit(&ctx, result);
}

async fn dispatch(command: Feature, ctx: &Context) -> CommandResult {
    match command {
        Feature::Rkp { command } => duck_rkp::run(command, ctx).await,
        Feature::DeviceIds { command } => duck_device_ids::run(command, ctx),
        Feature::TrickyStore { command } => duck_tricky_store::run(command, ctx).await,
        Feature::Artifacts { command } => artifacts::run(command, ctx),
    }
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
