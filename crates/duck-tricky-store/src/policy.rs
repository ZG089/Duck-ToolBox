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

// TEESimulator rules mirror `PATCH_RE`, `OSVER_RE` and `MODE_RE` in its WebUI
// (`module/webroot/js/domain/schema.js`), the reference client for `config.json`.

/// `(\d{4}|YYYY)-(0[1-9]|1[0-2]|MM)(-(0[1-9]|[12]\d|3[01]|DD))?`
fn tees_date(value: &str) -> bool {
    let number_in = |text: &str, max: u32| {
        digits(text, 2)
            && text
                .parse::<u32>()
                .is_ok_and(|number| (1..=max).contains(&number))
    };
    let pieces: Vec<&str> = value.split('-').collect();
    let (year, month, day) = match pieces.as_slice() {
        [year, month] => (*year, *month, None),
        [year, month, day] => (*year, *month, Some(*day)),
        _ => return false,
    };
    (year == "YYYY" || digits(year, 4))
        && (month == "MM" || number_in(month, 12))
        && day.is_none_or(|day| day == "DD" || number_in(day, 31))
}

pub(crate) fn tees_patch(value: &str) -> bool {
    matches!(value, "today" | "no" | "harvested" | "system_property") || tees_date(value)
}

pub(crate) fn tees_os_version(value: &str) -> bool {
    matches!(value, "harvested" | "system_property")
        || (value.split('.').count() <= 3
            && value
                .split('.')
                .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit())))
}

pub(crate) fn tees_mode(value: &str) -> bool {
    matches!(value, "patch" | "generation")
}

pub(crate) fn boolean(value: &str) -> bool {
    matches!(value, "true" | "false")
}

// OhMyKeymint rules mirror `validate_trust_config` in its `src/config.rs`.

pub(crate) fn omk_os_version(value: &str) -> bool {
    value == "auto" || (matches!(value.len(), 1 | 2) && digits(value, value.len()))
}

/// A real calendar-shaped `YYYY-MM-DD` (month 01-12, day 01-31).
fn iso_date(value: &str) -> bool {
    let pieces: Vec<&str> = value.split('-').collect();
    let [year, month, day] = pieces.as_slice() else {
        return false;
    };
    let in_range = |text: &str, max: u32| {
        digits(text, 2)
            && text
                .parse::<u32>()
                .is_ok_and(|number| (1..=max).contains(&number))
    };
    digits(year, 4) && in_range(month, 12) && in_range(day, 31)
}

pub(crate) fn omk_security_patch(value: &str) -> bool {
    matches!(value, "auto" | "latest") || iso_date(value)
}

pub(crate) fn omk_boot_patchlevel(value: &str) -> bool {
    omk_security_patch(value) || value.parse::<u32>().is_ok()
}

pub(crate) fn omk_vb(value: &str) -> bool {
    matches!(value, "auto" | "random")
        || (value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

#[cfg(test)]
mod tests {
    use super::{
        boolean, omk_boot_patchlevel, omk_os_version, omk_security_patch, omk_vb, tees_mode,
        tees_os_version, tees_patch, ts_day, ts_month,
    };

    #[test]
    fn tricky_store_patch_levels() {
        assert!(ts_month("202609") && ts_month("prop") && ts_month("no"));
        assert!(ts_day("20260905") && ts_day("2026-09-05"));
        assert!(!ts_month("2026090") && !ts_day("today"));
    }

    // Cases taken from TEESimulator's `module/webroot/tests/domain.test.mjs`.
    #[test]
    fn tee_simulator_levels() {
        for accepted in [
            "today",
            "no",
            "harvested",
            "system_property",
            "2024-01",
            "2024-12",
            "2024-01-15",
            "2024-12-31",
            "YYYY-MM",
            "YYYY-MM-05",
            "YYYY-MM-DD",
        ] {
            assert!(tees_patch(accepted), "{accepted}");
        }
        for rejected in [
            "2024",
            "2024-1",
            "2024-1-1",
            "yesterday",
            "",
            "2024-00",
            "2024-13",
            "2024-01-00",
            "2024-01-32",
            "2024-13-01",
            "MM-05",
            "YYYY-13-01",
        ] {
            assert!(!tees_patch(rejected), "{rejected}");
        }
        for accepted in [
            "harvested",
            "system_property",
            "16",
            "16.0",
            "16.0.0",
            "160000",
        ] {
            assert!(tees_os_version(accepted), "{accepted}");
        }
        for rejected in ["16.0.0.0", "v16", "", "16.", "no"] {
            assert!(!tees_os_version(rejected), "{rejected}");
        }
        assert!(tees_mode("patch") && tees_mode("generation") && !tees_mode("hybrid"));
    }

    #[test]
    fn oh_my_keymint_trust_values() {
        assert!(omk_security_patch("2026-09-05") && omk_security_patch("latest"));
        assert!(!omk_security_patch("2026-13-01") && !omk_security_patch("2026-09"));
        assert!(omk_boot_patchlevel("20260905") && omk_boot_patchlevel("auto"));
        assert!(omk_os_version("16") && omk_os_version("auto"));
        assert!(!omk_os_version("160") && !omk_os_version("16.0"));
        assert!(omk_vb(&"a".repeat(64)) && omk_vb("random"));
        assert!(!omk_vb("abc"));
        assert!(boolean("true") && !boolean("yes"));
    }
}
