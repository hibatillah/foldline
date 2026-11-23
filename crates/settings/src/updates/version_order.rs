//! Orders release tags against the running version.

/// Whether `latest_tag` (such as `v0.7.0`) is an update over `current_version` (such as `0.6.0-alpha.2`).
pub(super) fn is_newer(latest_tag: &str, current_version: &str) -> bool {
    let Some((latest, latest_suffix)) = parse_version(latest_tag) else {
        return false;
    };
    if latest_suffix.is_some() {
        return false;
    }
    let Some((current, current_suffix)) = parse_version(current_version) else {
        return false;
    };
    match latest.cmp(&current) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Equal => current_suffix.is_some(),
        std::cmp::Ordering::Less => false,
    }
}

/// Whether `version` reads as `1.2.3` with an optional `v` and suffix; a caller tells unreadable from not newer with it.
pub(super) fn is_readable(version: &str) -> bool {
    parse_version(version).is_some()
}

/// Splits `v1.2.3-suffix` into its three numbers and the optional suffix.
fn parse_version(text: &str) -> Option<([u64; 3], Option<&str>)> {
    let text = text.strip_prefix('v').unwrap_or(text);
    let (numbers, suffix) = match text.split_once('-') {
        Some((numbers, suffix)) => (numbers, Some(suffix)),
        None => (text, None),
    };
    let mut parts = numbers.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(([major, minor, patch], suffix))
}

#[cfg(test)]
mod tests {
    use super::{is_newer, is_readable};

    #[test]
    fn only_three_numbers_with_an_optional_v_and_suffix_are_readable() {
        for version in ["v0.7.0", "0.6.0", "v0.7.0-rc.1"] {
            assert!(is_readable(version), "{version}");
        }
        for version in ["latest", "", "v1.2", "v1.2.3.4", "dev"] {
            assert!(!is_readable(version), "{version}");
        }
    }

    #[test]
    fn a_higher_minor_is_newer() {
        assert!(is_newer("v0.7.0", "0.6.0"));
    }

    #[test]
    fn numbers_compare_numerically_not_as_text() {
        assert!(is_newer("0.10.0", "0.9.0"));
    }

    #[test]
    fn the_same_version_is_not_newer() {
        assert!(!is_newer("v0.6.0", "0.6.0"));
    }

    #[test]
    fn an_older_tag_is_not_newer() {
        assert!(!is_newer("v0.5.0", "0.6.0"));
    }

    #[test]
    fn a_release_is_newer_than_its_own_pre_release() {
        assert!(is_newer("v0.6.0", "0.6.0-alpha.2"));
    }

    #[test]
    fn a_tag_with_a_suffix_is_never_an_update() {
        assert!(!is_newer("v0.6.0-alpha.1", "0.6.0"));
    }

    #[test]
    fn a_tag_that_is_not_a_version_is_not_newer() {
        assert!(!is_newer("latest", "0.6.0"));
        assert!(!is_newer("", "0.6.0"));
        assert!(!is_newer("v1.2", "0.6.0"));
        assert!(!is_newer("v1.2.3.4", "0.6.0"));
    }
}
