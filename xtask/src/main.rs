//! Repository automation, run as `cargo xtask <task>`.
//!
//! Keeping these checks in Rust means CI and local runs share one implementation.

use std::path::PathBuf;

use anyhow::{Result, bail};

mod line_limit;

fn main() -> Result<()> {
    let task = std::env::args().nth(1);
    match task.as_deref() {
        Some("line-limit") => line_limit::run(&repo_root()),
        Some(other) => bail!("unknown xtask `{other}`; available tasks: line-limit"),
        None => bail!("usage: cargo xtask <task>; available tasks: line-limit"),
    }
}

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is `<repo>/xtask`.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives under the repository root")
        .to_path_buf()
}
