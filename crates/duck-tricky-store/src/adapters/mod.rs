//! Config adapters: one per keystore backend, behind a common trait.
//!
//! Adding support for another keystore module means adding a file here and one line in
//! [`for_backend`]; nothing else in the crate needs to change.

use anyhow::Result;
use duck_core::Sysroot;

use crate::model::{Backend, ConfigData, PolicySchema};

mod oh_my_keymint;
mod tee_simulator;
mod tricky_store;
mod tricky_store_legacy;

pub(crate) use tricky_store::TS_CONFIG_DIR;

/// Reads and writes one backend's on-disk configuration in the shared [`ConfigData`] shape.
pub(crate) trait ConfigAdapter {
    fn backend(&self) -> Backend;
    /// Device path of the backend's data directory.
    fn config_dir(&self) -> &'static str;
    /// Device path of the keybox this backend signs with.
    fn keybox_path(&self, _sysroot: &Sysroot) -> String {
        format!("{}/keybox.xml", self.config_dir())
    }
    fn policy_schema(&self) -> PolicySchema;
    fn read(&self, sysroot: &Sysroot) -> Result<ConfigData>;
    fn write(&self, sysroot: &Sysroot, config: &ConfigData) -> Result<()>;
}

pub(crate) fn for_backend(backend: Backend) -> Box<dyn ConfigAdapter> {
    match backend {
        Backend::TrickyStore => Box::new(tricky_store::TrickyStoreAdapter),
        Backend::TrickyStoreLegacy => Box::new(tricky_store_legacy::TrickyStoreLegacyAdapter),
        Backend::TeeSimulator => Box::new(tee_simulator::TeeSimulatorAdapter),
        Backend::OhMyKeymint => Box::new(oh_my_keymint::OhMyKeymintAdapter),
    }
}
