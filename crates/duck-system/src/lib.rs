//! Module lifecycle features Tricky Addon shipped for itself, generalised to the whole
//! toolbox: a device/system summary, update checks (the stable `updateJson` channel from
//! the KernelSU module guide and canary CI builds), installing an update through the root
//! manager, uninstalling, rebooting and opening links outside the WebView.

mod artifacts;
mod cli;
mod error;
mod files;
mod info;
mod update;

pub use cli::{Command, FEATURE, run};
pub use error::SystemError;
pub use info::{ModuleIdentity, SystemInfo};
pub use update::{Channel, UpdateInfo};
