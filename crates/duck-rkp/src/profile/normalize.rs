use std::path::{Component, Path, PathBuf};

use anyhow::Result;
use reqwest::Url;

use crate::error::RkpError;

use super::model::{
    DEFAULT_DICE_ISSUER, DEFAULT_DICE_SUBJECT, DeviceInfo, KeySource, ProfileData,
    default_output_path,
};

pub(crate) fn normalize_profile(mut profile: ProfileData) -> Result<ProfileData> {
    profile.key_source = normalize_key_source(&profile.key_source)?;
    normalize_device_info(&mut profile.device);
    profile.fingerprint.value = profile.fingerprint.value.trim().to_owned();
    profile.server_url = normalize_server_url(&profile.server_url)?;
    profile.output_path = normalize_output_path(&profile.output_path)?;
    Ok(profile)
}

fn normalize_key_source(key_source: &KeySource) -> Result<KeySource> {
    Ok(match key_source {
        KeySource::Unset => KeySource::Unset,
        KeySource::Seed { seed_hex } => {
            let seed_hex = seed_hex.trim().to_owned();
            if seed_hex.is_empty() {
                KeySource::Unset
            } else {
                KeySource::Seed { seed_hex }
            }
        }
        KeySource::HwKey {
            hw_key_hex,
            kdf_label,
        } => {
            let hw_key_hex = hw_key_hex.trim().to_owned();
            let kdf_label = kdf_label.trim().to_owned();

            if hw_key_hex.is_empty() {
                KeySource::Unset
            } else if kdf_label.is_empty() {
                return Err(RkpError::MissingKdfLabel.into());
            } else {
                KeySource::HwKey {
                    hw_key_hex,
                    kdf_label,
                }
            }
        }
    })
}

fn normalize_device_info(device: &mut DeviceInfo) {
    device.brand = device.brand.trim().to_owned();
    device.model = device.model.trim().to_owned();
    device.device = device.device.trim().to_owned();
    device.product = device.product.trim().to_owned();
    device.manufacturer = device.manufacturer.trim().to_owned();
    device.vb_state = device.vb_state.trim().to_ascii_lowercase();
    device.os_version = device.os_version.trim().to_owned();
    device.security_level = device.security_level.trim().to_ascii_lowercase();
    device.bootloader_state = device.bootloader_state.trim().to_ascii_lowercase();
    device.dice_issuer = non_blank_or(&device.dice_issuer, DEFAULT_DICE_ISSUER);
    device.dice_subject = non_blank_or(&device.dice_subject, DEFAULT_DICE_SUBJECT);
    device.vbmeta_digest = device
        .vbmeta_digest
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase);
}

fn non_blank_or(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_owned()
    } else {
        trimmed.to_owned()
    }
}

pub(crate) fn normalize_server_url(value: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(RkpError::MissingServerUrl.into());
    }

    let mut url =
        Url::parse(trimmed).map_err(|_| RkpError::InvalidServerUrl(trimmed.to_owned()))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(RkpError::InvalidServerUrl(trimmed.to_owned()).into());
    }

    url.set_query(None);
    url.set_fragment(None);

    let normalized_path = match url.path().trim_end_matches('/') {
        "" | "/" => "/v1".to_owned(),
        other => other.to_owned(),
    };
    url.set_path(&normalized_path);

    Ok(url.to_string())
}

pub(crate) fn normalize_output_path(value: &str) -> Result<String> {
    let normalized = value.trim().replace('\\', "/");
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        return Ok(default_output_path());
    }

    if trimmed.ends_with('/') {
        return Err(RkpError::InvalidOutputPath(trimmed.to_owned()).into());
    }

    let mut relative = PathBuf::new();
    for component in Path::new(trimmed).components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => relative.push(part),
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                return Err(RkpError::InvalidOutputPath(trimmed.to_owned()).into());
            }
        }
    }

    if relative.as_os_str().is_empty() || relative.file_name().is_none() {
        return Err(RkpError::InvalidOutputPath(trimmed.to_owned()).into());
    }

    Ok(relative.to_string_lossy().replace('\\', "/"))
}
