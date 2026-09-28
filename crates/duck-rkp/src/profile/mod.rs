//! RKP profile: device identity, key source and server settings used for provisioning.

pub mod detect;
mod model;
mod normalize;
mod store;
#[cfg(test)]
mod tests;

pub use model::*;
pub use store::{
    ResolvedProfile, RunOverrides, clear_profile, resolve_profile, save_profile, show_profile,
    validate_profile_name,
};
