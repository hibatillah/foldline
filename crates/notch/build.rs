include!("../../scripts/icon_resource.rs");

#[path = "../../scripts/release_version.rs"]
mod release_version;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=../../scripts/icon_resource.rs");
    println!("cargo:rerun-if-changed=../../scripts/release_version.rs");
    let version = release_version::current(&PathBuf::from("../.."));
    compile_icon_resource("../../assets/branding", "NOTCH_ICON", release_version::icon_for_version(version.as_deref()))
}
