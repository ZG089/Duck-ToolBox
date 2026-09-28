//! Adapters over the Android userspace and root managers.
//!
//! Everything here goes through documented, stable interfaces: `getprop`, `pm`, the
//! KernelSU/Magisk module directory layout and the manager-provided `resetprop`. Features
//! call these helpers instead of shelling out themselves, so a platform change is handled
//! in one place.

pub mod exec;
pub mod modules;
pub mod net;
pub mod packages;
pub mod props;
pub mod root;
