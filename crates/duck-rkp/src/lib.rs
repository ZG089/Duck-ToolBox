//! Remote Key Provisioning (RKP) workbench.
//!
//! Builds `AuthenticatedRequest` CSRs as specified by the AOSP IRemotelyProvisionedComponent
//! HAL (`hardware/interfaces/security/rkp`), talks to the RKP server and exports the returned
//! certificate chain as a Tricky Store style `keybox.xml`.

pub mod cbor;
pub mod cli;
pub mod cose;
pub mod crypto_kdf;
pub mod error;
mod handlers;
pub mod http;
pub mod keybox_xml;
mod output;
pub mod patch_level;
pub mod profile;
pub mod validate;
pub mod verify;

pub use cli::{Command, FEATURE, run};
