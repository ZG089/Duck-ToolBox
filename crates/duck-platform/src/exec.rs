use std::process::{Command, Output, Stdio};

use anyhow::{Context, Result, bail};

/// Runs a program and returns its stdout, failing on a non-zero exit status.
pub fn stdout(program: &str, args: &[&str]) -> Result<String> {
    let output = run(program, args)?;
    if !output.status.success() {
        bail!(
            "`{program} {}` exited with {}: {}",
            args.join(" "),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Like [`stdout`] but treats a missing binary or failure as "no output".
pub fn stdout_or_empty(program: &str, args: &[&str]) -> String {
    stdout(program, args).unwrap_or_default()
}

pub fn run(program: &str, args: &[&str]) -> Result<Output> {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .with_context(|| format!("run `{program}`"))
}

/// Returns the first candidate that exists and is a file, falling back to the bare name
/// so `PATH` lookup still applies (and tests can shadow the tool).
pub fn find_tool(name: &str, candidates: &[&str]) -> String {
    candidates
        .iter()
        .find(|candidate| std::path::Path::new(candidate).is_file())
        .map(|candidate| (*candidate).to_owned())
        .unwrap_or_else(|| name.to_owned())
}
