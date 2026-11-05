include!("../../scripts/icon_resource.rs");

#[path = "src/pre_release.rs"]
mod pre_release;

use std::path::Path;

/// The latest `v*` tag, when git runs and finds one.
fn latest_version_tag(workspace_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0", "--match", "v*"])
        .current_dir(workspace_root)
        .output()
        .ok()?;
    output.status.success().then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Sets `NOTCH_RELEASE_VERSION` from the latest pre-release tag of this crate version's series.
fn emit_local_release_version() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let workspace_root = Path::new(&manifest_dir).join("../..");
    // A `.git` file (worktree) is skipped: its refs live elsewhere and a missing label is acceptable.
    let git_dir = workspace_root.join(".git");
    if git_dir.is_dir() {
        for watched in ["HEAD", "refs/tags", "packed-refs"] {
            let path = git_dir.join(watched);
            if path.exists() {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
    let crate_version = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let label =
        latest_version_tag(&workspace_root).and_then(|tag| pre_release::pre_release_label(&tag, &crate_version));
    if let Some(label) = label {
        println!("cargo:rustc-env=NOTCH_RELEASE_VERSION={label}");
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=../../scripts/icon_resource.rs");
    // The release build sets the full version (0.5.0-alpha.1) that the sidebar shows; a local build derives it from git.
    println!("cargo:rerun-if-env-changed=NOTCH_RELEASE_VERSION");
    if std::env::var_os("NOTCH_RELEASE_VERSION").is_none() {
        emit_local_release_version();
    }
    compile_icon_resource("../../assets/branding", "settings.rc")
}
