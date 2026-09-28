use clap::{Parser, Subcommand};

use crate::artifacts::ArtifactCommand;

/// Duck ToolBox backend. Every subcommand prints exactly one JSON envelope on stdout.
#[derive(Debug, Parser)]
#[command(name = "duckd", version, about = "Duck ToolBox backend")]
pub struct Cli {
    /// Accepted for compatibility with the WebUI bridge; output is always JSON.
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Feature,
}

#[derive(Debug, Subcommand)]
pub enum Feature {
    /// Remote Key Provisioning workbench.
    Rkp {
        #[command(subcommand)]
        command: duck_rkp::Command,
    },
    /// Qualcomm Keymaster device ID provisioning.
    DeviceIds {
        #[command(subcommand)]
        command: duck_device_ids::Command,
    },
    /// Tricky Store / TEESimulator / OhMyKeymint manager.
    TrickyStore {
        #[command(subcommand)]
        command: duck_tricky_store::Command,
    },
    /// Generated artifacts and runtime paths.
    Artifacts {
        #[command(subcommand)]
        command: ArtifactCommand,
    },
}
