use clap::{Args, Subcommand};
use duck_core::{BoxFuture, ClapFeature, CommandResult, Context, FeatureInfo, IntoCommandResult};

use crate::{error::code_of, handlers, profile::DiceCurve};

pub static FEATURE: ClapFeature<Command> = ClapFeature::new(
    FeatureInfo {
        id: "rkp",
        summary: "Remote Key Provisioning workbench",
        contract: 1,
    },
    dispatch,
);

fn dispatch<'a>(command: Command, ctx: &'a Context) -> BoxFuture<'a, CommandResult> {
    Box::pin(run(command, ctx))
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show, save or clear the saved RKP profile.
    Profile {
        #[command(subcommand)]
        command: ProfileCommand,
    },
    /// Print the derived DICE key and effective settings.
    Info(InfoArgs),
    /// Build a CSR, verify it locally and submit it to the RKP server.
    Provision(ProvisionArgs),
    /// Provision one key and export the certificate chain as `keybox.xml`.
    Keybox(KeyboxArgs),
    /// Verify a CSR file offline.
    Verify(VerifyArgs),
}

#[derive(Debug, Subcommand)]
pub enum ProfileCommand {
    Show(ProfileArgs),
    Save(ProfileSaveArgs),
    Clear(ProfileArgs),
    /// Profile defaults read from this device's build properties (not saved).
    Detect,
}

#[derive(Debug, Args, Clone)]
pub struct ProfileArgs {
    #[arg(long)]
    pub profile: Option<String>,
}

#[derive(Debug, Args, Clone)]
pub struct ProfileSaveArgs {
    #[arg(long)]
    pub profile: Option<String>,
    /// Read the profile JSON from stdin.
    #[arg(long = "stdin-json")]
    pub stdin_json: bool,
}

#[derive(Debug, Args, Clone, Default)]
pub struct SharedRunArgs {
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, conflicts_with = "hw_key")]
    pub seed: Option<String>,
    #[arg(long, conflicts_with = "seed")]
    pub hw_key: Option<String>,
    #[arg(long)]
    pub kdf_label: Option<String>,
    #[arg(long, value_enum)]
    pub curve: Option<DiceCurve>,
    #[arg(long)]
    pub server_url: Option<String>,
}

#[derive(Debug, Args, Clone)]
pub struct InfoArgs {
    #[command(flatten)]
    pub shared: SharedRunArgs,
}

#[derive(Debug, Args, Clone)]
pub struct ProvisionArgs {
    #[command(flatten)]
    pub shared: SharedRunArgs,
    #[arg(long = "num-keys")]
    pub num_keys: Option<u32>,
}

#[derive(Debug, Args, Clone)]
pub struct KeyboxArgs {
    #[command(flatten)]
    pub shared: SharedRunArgs,
    #[arg(long)]
    pub output: Option<String>,
}

#[derive(Debug, Args, Clone)]
pub struct VerifyArgs {
    pub csr_file: String,
}

pub async fn run(command: Command, ctx: &Context) -> CommandResult {
    let paths = &ctx.paths;
    match command {
        Command::Profile { command } => match command {
            ProfileCommand::Show(args) => {
                handlers::profile::show(paths, &args).into_command("rkp.profile.show", code_of)
            }
            ProfileCommand::Save(args) => {
                handlers::profile::save(paths, &args).into_command("rkp.profile.save", code_of)
            }
            ProfileCommand::Clear(args) => {
                handlers::profile::clear(paths, &args).into_command("rkp.profile.clear", code_of)
            }
            ProfileCommand::Detect => {
                handlers::profile::detect().into_command("rkp.profile.detect", code_of)
            }
        },
        Command::Info(args) => {
            handlers::info::run(paths, &args.shared).into_command("rkp.info", code_of)
        }
        Command::Provision(args) => handlers::provision::run(paths, &args)
            .await
            .into_command("rkp.provision", code_of),
        Command::Keybox(args) => handlers::keybox::run(paths, &args)
            .await
            .into_command("rkp.keybox", code_of),
        Command::Verify(args) => handlers::verify(paths, &args).into_command("rkp.verify", code_of),
    }
}
