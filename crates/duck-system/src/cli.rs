use anyhow::Result;
use clap::{Args, Subcommand};
use duck_core::{
    BoxFuture, ClapFeature, CommandResult, Context, FeatureInfo, IntoCommandResult, log,
};
use duck_platform::{exec, net, root};
use serde_json::json;

use crate::{
    SystemError, artifacts,
    error::code_of,
    files, info,
    update::{self, Channel},
};

pub static FEATURE: ClapFeature<Command> = ClapFeature::new(
    FeatureInfo {
        id: "system",
        summary: "Module lifecycle: system summary, updates, uninstall, reboot and links",
        contract: 1,
    },
    dispatch,
);

fn dispatch<'a>(command: Command, ctx: &'a Context) -> BoxFuture<'a, CommandResult> {
    Box::pin(run(command, ctx))
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Module, root manager and device summary.
    Info,
    /// Check for or install a module update.
    Update {
        #[command(subcommand)]
        command: UpdateCommand,
    },
    /// Generated output files and runtime paths.
    Artifacts,
    /// List a directory for the WebUI file picker.
    Files(FilesArgs),
    /// The most recent command log entries.
    Log(LogArgs),
    /// Open an http(s) link in the system browser.
    OpenUrl(OpenUrlArgs),
    /// Mark Duck ToolBox for removal on the next boot.
    Uninstall,
    /// Reboot the device.
    Reboot,
}

#[derive(Debug, Subcommand)]
pub enum UpdateCommand {
    Check(ChannelArgs),
    Install(ChannelArgs),
}

#[derive(Debug, Args, Clone)]
pub struct ChannelArgs {
    #[arg(long, value_enum, default_value = "stable")]
    pub channel: Channel,
}

#[derive(Debug, Args, Clone)]
pub struct FilesArgs {
    #[arg(long, default_value = "/storage/emulated/0/Download")]
    pub path: String,
    /// Only list files with this extension (directories are always listed).
    #[arg(long, default_value = "")]
    pub extension: String,
}

#[derive(Debug, Args, Clone)]
pub struct LogArgs {
    #[arg(long, default_value_t = 50)]
    pub limit: usize,
}

#[derive(Debug, Args, Clone)]
pub struct OpenUrlArgs {
    pub url: String,
}

pub async fn run(command: Command, ctx: &Context) -> CommandResult {
    match command {
        Command::Info => {
            info::collect(&ctx.paths, &ctx.sysroot).into_command("system.info", code_of)
        }
        Command::Update { command } => match command {
            UpdateCommand::Check(args) => update::check(&ctx.paths, args.channel)
                .await
                .into_command("system.update.check", code_of),
            UpdateCommand::Install(args) => update::install(ctx, args.channel)
                .await
                .into_command("system.update.install", code_of),
        },
        Command::Artifacts => artifacts::list(&ctx.paths).into_command("system.artifacts", code_of),
        Command::Files(args) => {
            files::list(ctx, &args.path, &args.extension).into_command("system.files", code_of)
        }
        Command::Log(args) => log::tail(&ctx.paths, args.limit)
            .map(|entries| json!({ "entries": entries }))
            .into_command("system.log", code_of),
        Command::OpenUrl(args) => open_url(&args.url)
            .map(|()| json!({ "url": args.url }))
            .into_command("system.open-url", code_of),
        Command::Uninstall => uninstall(ctx)
            .map(|id| json!({ "module_id": id, "reboot_required": true }))
            .into_command("system.uninstall", code_of),
        Command::Reboot => reboot()
            .map(|()| json!({ "rebooting": true }))
            .into_command("system.reboot", code_of),
    }
}

/// The WebView cannot leave the manager, so links go through the activity manager.
fn open_url(url: &str) -> Result<()> {
    if !net::is_web_url(url) {
        return Err(SystemError::InvalidUrl(url.to_owned()).into());
    }
    exec::stdout(
        "am",
        &["start", "-a", "android.intent.action.VIEW", "-d", url],
    )
    .map(|_| ())
}

fn uninstall(ctx: &Context) -> Result<String> {
    let manager = root::detect(&ctx.sysroot).ok_or(SystemError::NoRootManager)?;
    let id = info::module_identity(&ctx.paths)?.id;
    manager.uninstall_module(&ctx.sysroot, &id)?;
    Ok(id)
}

fn reboot() -> Result<()> {
    exec::stdout("svc", &["power", "reboot"])
        .or_else(|_| exec::stdout("reboot", &[]))
        .map(|_| ())
}
