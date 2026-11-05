//! Picks the pre-release label of a local build; shared with `build.rs`, which includes this file as a module.

/// Returns the tag without its `v` when it is a pre-release of `crate_version` (`v0.5.0-alpha.1` for `0.5.0`).
pub fn pre_release_label(tag: &str, crate_version: &str) -> Option<String> {
    let version = tag.trim().strip_prefix('v')?;
    let (core, suffix) = version.split_once('-')?;
    (core == crate_version && !suffix.is_empty()).then(|| version.to_string())
}

#[cfg(test)]
mod tests {
    use super::pre_release_label;

    #[test]
    fn accepts_pre_release_of_the_same_series() {
        assert_eq!(pre_release_label("v0.5.0-alpha.1\n", "0.5.0").as_deref(), Some("0.5.0-alpha.1"));
        assert_eq!(pre_release_label("v0.5.0-rc.2", "0.5.0").as_deref(), Some("0.5.0-rc.2"));
    }

    #[test]
    fn rejects_stable_tags_and_other_series() {
        assert_eq!(pre_release_label("v0.5.0", "0.5.0"), None);
        assert_eq!(pre_release_label("v0.4.0-alpha.1", "0.5.0"), None);
        assert_eq!(pre_release_label("0.5.0-alpha.1", "0.5.0"), None);
        assert_eq!(pre_release_label("v0.5.0-", "0.5.0"), None);
        assert_eq!(pre_release_label("", "0.5.0"), None);
    }
}
