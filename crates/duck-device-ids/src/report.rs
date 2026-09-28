use anyhow::{Context, Result};
use duck_core::{AppPaths, fs::write_bytes_atomic};
use serde::Serialize;

use crate::{DeviceIdsProfile, ProvisionedId, qseecom::SessionInfo};

#[derive(Debug, Serialize)]
struct DeviceIdsReport<'a> {
    profile: &'a DeviceIdsProfile,
    ids: &'a [ProvisionedId],
    dry_run: bool,
    loaded_library: &'a Option<String>,
    ta_api_version: &'a Option<String>,
    ta_version: &'a Option<String>,
    response_hex: &'a Option<String>,
    command_hex: String,
}

pub(crate) fn write(
    paths: &AppPaths,
    profile: &DeviceIdsProfile,
    ids: &[ProvisionedId],
    session: &SessionInfo,
    command: &[u8],
) -> Result<String> {
    let run_dir = duck_core::fs::create_unique_dir(&paths.outputs_dir, "device-ids")
        .with_context(|| format!("create report dir under {}", paths.outputs_dir.display()))?;
    let report_path = run_dir.join("device_ids_report.json");

    let report = DeviceIdsReport {
        profile,
        ids,
        dry_run: profile.dry_run,
        loaded_library: &session.loaded_library,
        ta_api_version: &session.ta_api_version,
        ta_version: &session.ta_version,
        response_hex: &session.response_hex,
        command_hex: hex::encode(command),
    };
    let bytes = serde_json::to_vec_pretty(&report).context("serialize device ID report")?;
    write_bytes_atomic(&report_path, &bytes)
        .with_context(|| format!("write {}", report_path.display()))?;

    Ok(report_path.display().to_string())
}
