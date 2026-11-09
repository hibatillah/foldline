include!("../../scripts/icon_resource.rs");

#[path = "../../scripts/release_version.rs"]
mod release_version;

use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=../../scripts/icon_resource.rs");
    println!("cargo:rerun-if-changed=../../scripts/release_version.rs");
    let manifest_directory = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let workspace_root = Path::new(&manifest_directory).join("../..");
    let version = release_version::current(&workspace_root);
    if let Some(version) = &version {
        println!("cargo:rustc-env=NOTCH_RELEASE_VERSION={version}");
    }
    compile_icon_resource("../../assets/branding", "1", release_version::icon_for_version(version.as_deref()))
}
