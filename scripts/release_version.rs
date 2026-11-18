//! The release version a build belongs to and the app icon it gets; a module of both build scripts (`#[path]`) and of the
//! settings crate's tests, which run the tests below.
// The build scripts use different parts of it, and the settings crate's tests include it whole.
#![allow(dead_code)]

/// The icon of a pre-release build (alpha, beta, rc).
pub const DEV_ICON: &str = "tile-dev.ico";
/// The icon of a stable build.
pub const STABLE_ICON: &str = "tile-stable.ico";

/// Returns the tag without its `v` when it is a pre-release of `crate_version` (`v0.5.0-alpha.1` for `0.5.0`).
pub fn pre_release_label(tag: &str, crate_version: &str) -> Option<String> {
    let version = tag.trim().strip_prefix('v')?;
    let (core, suffix) = version.split_once('-')?;
    (core == crate_version && !suffix.is_empty()).then(|| version.to_string())
}

/// The icon file for a version: the dev tile when it carries a pre-release suffix, the stable tile otherwise.
pub fn icon_for_version(version: Option<&str>) -> &'static str {
    let version = version.map(|value| value.trim().trim_start_matches('v')).unwrap_or_default();
    match version.split_once('-') {
        Some((_, suffix)) if !suffix.is_empty() => DEV_ICON,
        _ => STABLE_ICON,
    }
}

/// The `VERSIONINFO` resource text that names a program in Task Manager (its Processes tab shows `file_description`).
///
/// `version` may carry a pre-release suffix (`0.5.0-alpha.4`); the numeric `a,b,c,d` form takes its core and a last 0.
pub fn version_info_resource(
    file_description: &str,
    product_name: &str,
    version: &str,
    original_filename: &str,
) -> String {
    let version = version.trim().trim_start_matches('v');
    let core = version.split_once('-').map_or(version, |(core, _)| core);
    let mut numbers = core.split('.').map(|part| part.parse::<u16>().unwrap_or(0));
    let numeric = (0..4).map(|_| numbers.next().unwrap_or(0).to_string()).collect::<Vec<_>>().join(",");
    let quoted = |value: &str| value.replace('"', "\"\"");
    let (file_description, product_name) = (quoted(file_description), quoted(product_name));
    let (version, original_filename) = (quoted(version), quoted(original_filename));
    let internal_name = &original_filename;
    format!(
        concat!(
            "1 VERSIONINFO\n",
            " FILEVERSION {numeric}\n",
            " PRODUCTVERSION {numeric}\n",
            " FILEOS 0x40004\n",
            " FILETYPE 1\n",
            "{{\n",
            " BLOCK \"StringFileInfo\"\n",
            " {{\n",
            "  BLOCK \"040904B0\"\n",
            "  {{\n",
            "   VALUE \"FileDescription\", \"{file_description}\"\n",
            "   VALUE \"FileVersion\", \"{version}\"\n",
            "   VALUE \"InternalName\", \"{internal_name}\"\n",
            "   VALUE \"OriginalFilename\", \"{original_filename}\"\n",
            "   VALUE \"ProductName\", \"{product_name}\"\n",
            "   VALUE \"ProductVersion\", \"{version}\"\n",
            "  }}\n",
            " }}\n",
            " BLOCK \"VarFileInfo\"\n",
            " {{\n",
            "  VALUE \"Translation\", 0x0409, 0x04B0\n",
            " }}\n",
            "}}\n",
        ),
        numeric = numeric,
        file_description = file_description,
        version = version,
        internal_name = internal_name,
        original_filename = original_filename,
        product_name = product_name,
    )
}

/// The latest `v*` tag, when git runs and finds one.
fn latest_version_tag(workspace_root: &std::path::Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["describe", "--tags", "--abbrev=0", "--match", "v*"])
        .current_dir(workspace_root)
        .output()
        .ok()?;
    output.status.success().then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// The version this build is for, and the cargo rebuild triggers that keep it fresh.
///
/// The release build sets `NOTCH_RELEASE_VERSION` (`0.5.0-alpha.1`); a local build derives it from the latest
/// pre-release tag of this crate version's series, and has none when there is no such tag.
pub fn current(workspace_root: &std::path::Path) -> Option<String> {
    println!("cargo:rerun-if-env-changed=NOTCH_RELEASE_VERSION");
    if let Some(value) = std::env::var("NOTCH_RELEASE_VERSION").ok().filter(|value| !value.trim().is_empty()) {
        return Some(value);
    }
    // A `.git` file (worktree) is skipped: its refs live elsewhere and a missing label is acceptable.
    let git_directory = workspace_root.join(".git");
    if git_directory.is_dir() {
        for watched in ["HEAD", "refs/tags", "packed-refs"] {
            let path = git_directory.join(watched);
            if path.exists() {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
    let crate_version = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    latest_version_tag(workspace_root).and_then(|tag| pre_release_label(&tag, &crate_version))
}

#[cfg(test)]
mod tests {
    use super::{DEV_ICON, STABLE_ICON, icon_for_version, pre_release_label, version_info_resource};

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

    #[test]
    fn pre_release_versions_get_the_dev_icon() {
        for version in ["0.5.0-alpha.1", "v0.5.0-beta.2", "0.5.0-rc.1", " 0.5.0-rc.1\n"] {
            assert_eq!(icon_for_version(Some(version)), DEV_ICON, "{version}");
        }
    }

    #[test]
    fn stable_or_unknown_versions_get_the_stable_icon() {
        for version in [Some("0.5.0"), Some("v0.5.0"), Some("0.5.0-"), Some(""), None] {
            assert_eq!(icon_for_version(version), STABLE_ICON, "{version:?}");
        }
    }

    #[test]
    fn version_info_names_the_program_and_carries_the_four_number_version() {
        let text = version_info_resource("Foldline Settings", "Foldline", "0.5.0-alpha.4", "notch-settings.exe");
        for expected in [
            "FILEVERSION 0,5,0,0",
            "PRODUCTVERSION 0,5,0,0",
            "VALUE \"FileDescription\", \"Foldline Settings\"",
            "VALUE \"ProductName\", \"Foldline\"",
            "VALUE \"FileVersion\", \"0.5.0-alpha.4\"",
            "VALUE \"ProductVersion\", \"0.5.0-alpha.4\"",
            "VALUE \"InternalName\", \"notch-settings.exe\"",
            "VALUE \"OriginalFilename\", \"notch-settings.exe\"",
        ] {
            assert!(text.contains(expected), "{expected}\n{text}");
        }
    }

    #[test]
    fn version_info_of_a_stable_version_has_no_suffix() {
        let text = version_info_resource("Foldline", "Foldline", "v0.5.0", "notch.exe");
        assert!(text.contains("VALUE \"FileVersion\", \"0.5.0\""), "{text}");
        assert!(text.contains("FILEVERSION 0,5,0,0"), "{text}");
    }
}
