use std::io::{self, Read};

use anyhow::{Context as _, Result};
use clap::{Args, Subcommand};
use duck_core::{BoxFuture, ClapFeature, CommandResult, Context, FeatureInfo, IntoCommandResult};
use serde::de::DeserializeOwned;

use crate::{
    auto, entry, error::code_of, exclude, keybox, model::SaveRequest, props, providers, service,
    state::KeyboxProvider, xposed,
};

pub static FEATURE: ClapFeature<Command> = ClapFeature::new(
    FeatureInfo {
        id: "tricky-store",
        summary: "Tricky Store / TEESimulator / OhMyKeymint manager",
        contract: 1,
    },
    dispatch,
)
.with_status(service::status_line);

fn dispatch<'a>(command: Command, ctx: &'a Context) -> BoxFuture<'a, CommandResult> {
    Box::pin(run(command, ctx))
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Detected backends, targets, packages, keybox and prop status.
    Status,
    /// Persist targets, policies, system apps and the auto-add toggle (JSON on stdin).
    Save(StdinArgs),
    /// Keybox management for the active backend.
    Keybox {
        #[command(subcommand)]
        command: KeyboxCommand,
    },
    /// App-list helpers used by the target picker.
    Apps {
        #[command(subcommand)]
        command: AppsCommand,
    },
    /// Prop handler and Verified Boot hash (JSON on stdin).
    Props(StdinArgs),
    /// Apply auto-target additions (run on boot).
    AutoApply,
    /// WebUI entry on the keystore module itself.
    Entry {
        #[command(subcommand)]
        command: EntryCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum EntryCommand {
    Status,
    Enable,
    Disable,
    /// Restore the saved setting (run on boot).
    Apply,
    /// Remove every link this feature created (run on uninstall).
    Remove,
}

#[derive(Debug, Subcommand)]
pub enum KeyboxCommand {
    /// Install a keybox from a local file path.
    Install(SourceArgs),
    /// Install keybox XML sent on stdin as `{ "content": "..." }` (browser file picker).
    Import(StdinArgs),
    /// Install the bundled AOSP software keybox.
    SetAosp,
    /// Generate and install a self-signed "unknown" keybox.
    Generate,
    /// Download a keybox from a URL and install it.
    Fetch(FetchArgs),
    /// Manage custom keybox providers.
    Providers {
        #[command(subcommand)]
        command: ProviderCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum ProviderCommand {
    List,
    Save(StdinArgs),
    Reset,
    Import(SourceArgs),
    /// Merge an export file sent on stdin as `{ "content": "..." }`.
    ImportContent(StdinArgs),
    Export(ExportArgs),
}

#[derive(Debug, Subcommand)]
pub enum AppsCommand {
    /// Installed Xposed modules.
    Xposed,
    /// Magisk DenyList packages (empty on other managers).
    Denylist,
    /// Root managers and apps that never check bootloader state.
    Unnecessary(RefreshArgs),
}

#[derive(Debug, Args, Clone)]
pub struct StdinArgs {
    #[arg(long = "stdin-json")]
    pub stdin_json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct SourceArgs {
    pub source: String,
}

#[derive(Debug, Args, Clone)]
pub struct FetchArgs {
    #[arg(long)]
    pub url: String,
    #[arg(long, default_value = "")]
    pub decode: String,
}

#[derive(Debug, Args, Clone)]
pub struct ExportArgs {
    #[arg(long)]
    pub path: Option<String>,
}

#[derive(Debug, Args, Clone)]
pub struct RefreshArgs {
    #[arg(long)]
    pub refresh: bool,
}

/// File contents chosen in the WebView's own file picker, which exposes no path.
#[derive(Debug, serde::Deserialize)]
struct ContentRequest {
    content: String,
}

pub async fn run(command: Command, ctx: &Context) -> CommandResult {
    match command {
        Command::Status => service::status(ctx).into_command("tricky-store.status", code_of),
        Command::Save(args) => {
            let request = read_stdin_json::<SaveRequest>(args.stdin_json);
            request
                .and_then(|request| service::save(ctx, request))
                .into_command("tricky-store.save", code_of)
        }
        Command::Keybox { command } => keybox_command(command, ctx).await,
        Command::Apps { command } => apps_command(command, ctx).await,
        Command::Props(args) => {
            let request = read_stdin_json::<props::PropRequest>(args.stdin_json);
            request
                .and_then(|request| props::save(ctx, request))
                .into_command("tricky-store.props", code_of)
        }
        Command::AutoApply => auto::apply(ctx).into_command("tricky-store.auto-apply", code_of),
        Command::Entry { command } => match command {
            EntryCommand::Status => entry::status(ctx),
            EntryCommand::Enable => entry::set(ctx, true),
            EntryCommand::Disable => entry::set(ctx, false),
            EntryCommand::Apply => entry::apply(ctx),
            EntryCommand::Remove => entry::remove_all(ctx).and_then(|()| entry::status(ctx)),
        }
        .into_command("tricky-store.entry", code_of),
    }
}

async fn keybox_command(command: KeyboxCommand, ctx: &Context) -> CommandResult {
    match command {
        KeyboxCommand::Install(args) => {
            install_local(ctx, &args.source).into_command("tricky-store.keybox.install", code_of)
        }
        KeyboxCommand::Import(args) => read_stdin_json::<ContentRequest>(args.stdin_json)
            .and_then(|request| keybox::install(ctx, &request.content, "local"))
            .into_command("tricky-store.keybox.import", code_of),
        KeyboxCommand::SetAosp => keybox::install(ctx, keybox::AOSP_KEYBOX, "aosp")
            .into_command("tricky-store.keybox.set-aosp", code_of),
        KeyboxCommand::Generate => {
            generate(ctx).into_command("tricky-store.keybox.generate", code_of)
        }
        KeyboxCommand::Fetch(args) => fetch(ctx, &args)
            .await
            .into_command("tricky-store.keybox.fetch", code_of),
        KeyboxCommand::Providers { command } => provider_command(command, ctx),
    }
}

fn provider_command(command: ProviderCommand, ctx: &Context) -> CommandResult {
    match command {
        ProviderCommand::List => {
            providers::list(ctx).into_command("tricky-store.keybox.providers.list", code_of)
        }
        ProviderCommand::Save(args) => read_stdin_json::<Vec<KeyboxProvider>>(args.stdin_json)
            .and_then(|entries| providers::save(ctx, entries))
            .into_command("tricky-store.keybox.providers.save", code_of),
        ProviderCommand::Reset => {
            providers::reset(ctx).into_command("tricky-store.keybox.providers.reset", code_of)
        }
        ProviderCommand::Import(args) => providers::import(ctx, &args.source)
            .into_command("tricky-store.keybox.providers.import", code_of),
        ProviderCommand::ImportContent(args) => read_stdin_json::<ContentRequest>(args.stdin_json)
            .and_then(|request| providers::import_content(ctx, &request.content))
            .into_command("tricky-store.keybox.providers.import", code_of),
        ProviderCommand::Export(args) => providers::export(ctx, args.path.as_deref())
            .map(|path| serde_json::json!({ "path": path }))
            .into_command("tricky-store.keybox.providers.export", code_of),
    }
}

async fn apps_command(command: AppsCommand, ctx: &Context) -> CommandResult {
    match command {
        AppsCommand::Xposed => xposed::xposed_modules(ctx)
            .map(|modules| serde_json::json!({ "packages": modules }))
            .into_command("tricky-store.apps.xposed", code_of),
        AppsCommand::Denylist => {
            anyhow::Ok(serde_json::json!({ "packages": exclude::denylist(ctx) }))
                .into_command("tricky-store.apps.denylist", code_of)
        }
        AppsCommand::Unnecessary(args) => exclude::load(ctx, args.refresh)
            .await
            .map(|(packages, source)| serde_json::json!({ "packages": packages, "source": source }))
            .into_command("tricky-store.apps.unnecessary", code_of),
    }
}

fn install_local(ctx: &Context, source: &str) -> Result<keybox::KeyboxInstallData> {
    let host = ctx.sysroot.path(source.trim());
    let content =
        std::fs::read_to_string(&host).with_context(|| format!("read keybox {source}"))?;
    keybox::install(ctx, &content, "local")
}

fn generate(ctx: &Context) -> Result<keybox::KeyboxInstallData> {
    let xml = keybox::generate::unknown_keybox()?;
    keybox::install(ctx, &xml, "generated")
}

async fn fetch(ctx: &Context, args: &FetchArgs) -> Result<keybox::KeyboxInstallData> {
    let content = keybox::fetch(&args.url, &args.decode).await?;
    keybox::install(ctx, &content, "url")
}

fn read_stdin_json<T: DeserializeOwned>(stdin_json: bool) -> Result<T> {
    anyhow::ensure!(stdin_json, "this command requires `--stdin-json`");
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .context("read JSON from stdin")?;
    serde_json::from_str(&input).context("parse JSON from stdin")
}
