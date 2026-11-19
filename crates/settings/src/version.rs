//! The version shown in the sidebar header.

/// The full release version set by the release build (`1.0.0-alpha.1`), or the crate version when it is not set.
pub fn displayed_version() -> &'static str {
    format_version(option_env!("FOLDLINE_RELEASE_VERSION"), env!("CARGO_PKG_VERSION"))
}

/// Picks the release version without its leading `v`; an unset or blank value falls back to the crate version.
fn format_version<'a>(release_version: Option<&'a str>, crate_version: &'a str) -> &'a str {
    match release_version.map(|value| value.trim().trim_start_matches('v')) {
        Some(value) if !value.is_empty() => value,
        _ => crate_version,
    }
}

#[cfg(test)]
mod tests {
    use super::format_version;

    #[test]
    fn uses_release_version_with_suffix() {
        assert_eq!(format_version(Some("1.0.0-alpha.1"), "1.0.0"), "1.0.0-alpha.1");
    }

    #[test]
    fn strips_leading_v() {
        assert_eq!(format_version(Some("v1.0.0-rc.2"), "1.0.0"), "1.0.0-rc.2");
    }

    #[test]
    fn falls_back_when_unset_or_blank() {
        assert_eq!(format_version(None, "1.0.0"), "1.0.0");
        assert_eq!(format_version(Some(""), "1.0.0"), "1.0.0");
        assert_eq!(format_version(Some("v"), "1.0.0"), "1.0.0");
    }
}
