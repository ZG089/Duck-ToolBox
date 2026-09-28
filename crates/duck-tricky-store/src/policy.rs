//! Policy validation. Each backend attaches these validators to its schema fields.

use crate::{
    error::TrickyError,
    model::{Policy, PolicySchema},
};

/// Keeps only the schema's keys, trims values, drops blanks and validates the rest.
pub(crate) fn sanitize(schema: &PolicySchema, policy: Policy) -> Result<Policy, TrickyError> {
    let mut clean = Policy::new();
    for field in &schema.default_policy {
        let Some(value) = policy.get(&field.key).map(|value| value.trim()) else {
            continue;
        };
        if value.is_empty() {
            continue;
        }
        if let Some(validator) = field.validator
            && !validator(value)
        {
            return Err(TrickyError::InvalidPolicy {
                field: field.key.clone(),
                reason: field
                    .hint
                    .clone()
                    .unwrap_or_else(|| "unsupported value".into()),
            });
        }
        clean.insert(field.key.clone(), value.to_owned());
    }
    Ok(clean)
}

fn digits(value: &str, len: usize) -> bool {
    value.len() == len && value.bytes().all(|byte| byte.is_ascii_digit())
}

/// `YYYY-MM-DD` or `YYYY-MM`, where Tricky Store also accepts dashed dates.
fn dashed_date(value: &str, allow_tokens: bool) -> bool {
    let part =
        |text: &str, len: usize, token: &str| digits(text, len) || (allow_tokens && text == token);
    let pieces: Vec<&str> = value.split('-').collect();
    match pieces.as_slice() {
        [year, month] => part(year, 4, "YYYY") && part(month, 2, "MM"),
        [year, month, day] => part(year, 4, "YYYY") && part(month, 2, "MM") && part(day, 2, "DD"),
        _ => false,
    }
}

pub(crate) fn ts_month(value: &str) -> bool {
    matches!(value, "prop" | "no") || digits(value, 6) || dashed_date(value, false)
}

pub(crate) fn ts_day(value: &str) -> bool {
    matches!(value, "prop" | "no") || digits(value, 8) || dashed_date(value, false)
}

pub(crate) fn tees_patch(value: &str) -> bool {
    matches!(value, "no" | "today" | "harvested" | "system_property") || dashed_date(value, true)
}

pub(crate) fn tees_os_version(value: &str) -> bool {
    matches!(value, "no" | "harvested" | "system_property")
        || (value.split('.').count() <= 3
            && value
                .split('.')
                .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit())))
}

pub(crate) fn omk_os_version(value: &str) -> bool {
    value == "auto" || (!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
}

pub(crate) fn omk_security_patch(value: &str) -> bool {
    matches!(value, "auto" | "latest") || (value.len() == 10 && dashed_date(value, false))
}

pub(crate) fn omk_vb(value: &str) -> bool {
    matches!(value, "auto" | "random")
        || (value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

#[cfg(test)]
mod tests {
    use super::{omk_security_patch, omk_vb, tees_os_version, tees_patch, ts_day, ts_month};

    #[test]
    fn tricky_store_patch_levels() {
        assert!(ts_month("202609") && ts_month("prop") && ts_month("no"));
        assert!(ts_day("20260905") && ts_day("2026-09-05"));
        assert!(!ts_month("2026090") && !ts_day("today"));
    }

    #[test]
    fn tee_simulator_levels() {
        assert!(tees_patch("today") && tees_patch("YYYY-MM-05") && tees_patch("2026-09"));
        assert!(tees_os_version("16") && tees_os_version("16.0.0") && tees_os_version("harvested"));
        assert!(!tees_patch("20260905") && !tees_os_version("16.0.0.1"));
    }

    #[test]
    fn oh_my_keymint_trust_values() {
        assert!(omk_security_patch("2026-09-05") && omk_security_patch("latest"));
        assert!(omk_vb(&"a".repeat(64)) && omk_vb("random"));
        assert!(!omk_security_patch("2026-09") && !omk_vb("abc"));
    }
}
