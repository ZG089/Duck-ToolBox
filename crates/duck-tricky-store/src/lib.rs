//! Multi-backend keystore-spoofing manager.
//!
//! This is Duck ToolBox's port of Tricky Addon (Update Target List). It manages whichever
//! keystore module is installed through a common [`adapters::ConfigAdapter`] trait: Tricky
//! Store (`config.ini` and the legacy `target.txt`), TEESimulator (`config.json`) and
//! OhMyKeymint (`config.toml` + `injector.toml`). Every backend-specific detail lives behind
//! that trait, so new backends slot in without touching the shared logic.

mod adapters;
pub mod auto;
mod detect;
pub mod entry;
pub mod error;
pub mod exclude;
pub mod files;
pub mod keybox;
pub mod model;
mod policy;
pub mod props;
pub mod providers;
mod service;
pub mod state;
mod targets;
pub mod xposed;

pub mod cli;

pub use cli::{Command, FEATURE, run};
pub use detect::{detect_active, detect_all};
pub use error::TrickyError;
pub use service::{SaveData, StatusData, save, status};

#[cfg(test)]
mod tests;
