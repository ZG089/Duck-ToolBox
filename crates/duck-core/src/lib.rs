//! Feature-agnostic runtime shared by every Duck ToolBox crate.
//!
//! Features depend on this crate, never on each other. Anything that knows about a
//! specific tool (RKP, Tricky Store, ...) belongs in that tool's crate instead.

pub mod command;
pub mod envelope;
pub mod feature;
pub mod fs;
pub mod log;
pub mod paths;
pub mod sysroot;

pub use command::{
    CommandFailure, CommandOutput, CommandResult, Context, Failure, IntoCommandResult,
    internal_error,
};
pub use feature::{BoxFuture, ClapFeature, Feature, FeatureInfo};
pub use paths::AppPaths;
pub use sysroot::Sysroot;

/// Seconds since the Unix epoch, or `0` if the clock is before 1970.
pub fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
