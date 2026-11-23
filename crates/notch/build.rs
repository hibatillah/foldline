include!("../../scripts/icon_resource.rs");

#[path = "../../scripts/release_version.rs"]
mod release_version;

#[path = "../../scripts/sparse_identity.rs"]
mod sparse_identity;

const SPARSE_MANIFEST: &str = "../../installer/sparse/AppxManifest.xml";

/// Writes the exe manifest with the sparse package's `<msix>` element and returns the resource line that embeds it.
fn exe_manifest_resource() -> Result<String, Box<dyn Error>> {
    println!("cargo:rerun-if-changed={SPARSE_MANIFEST}");
    let identity = sparse_identity::read_identity(&fs::read_to_string(SPARSE_MANIFEST)?)
        .ok_or("installer/sparse/AppxManifest.xml has no Identity Name and Publisher or no Application Id")?;
    let path = PathBuf::from(env::var_os("OUT_DIR").ok_or("OUT_DIR is missing")?).join("foldline.exe.manifest");
    fs::write(&path, sparse_identity::exe_manifest(&identity))?;
    // 1 is the manifest id of an executable and 24 is RT_MANIFEST; rc.exe reads backslashes as escapes.
    Ok(format!("1 24 \"{}\"\n", path.to_string_lossy().replace('\\', "/")))
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=../../scripts/icon_resource.rs");
    println!("cargo:rerun-if-changed=../../scripts/release_version.rs");
    println!("cargo:rerun-if-changed=../../scripts/sparse_identity.rs");
    let version = release_version::current(&PathBuf::from("../.."));
    let version_text = version.clone().unwrap_or_else(|| env::var("CARGO_PKG_VERSION").unwrap_or_default());
    let version_info = release_version::version_info_resource("Foldline", "Foldline", &version_text, "foldline.exe");
    let resources = format!("{version_info}\n{}", exe_manifest_resource()?);
    compile_icon_resource(
        "../../assets/branding",
        "FOLDLINE_ICON",
        release_version::icon_for_version(version.as_deref()),
        &resources,
    )
}
