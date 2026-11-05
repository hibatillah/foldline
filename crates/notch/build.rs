include!("../../scripts/icon_resource.rs");

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=../../scripts/icon_resource.rs");
    compile_icon_resource("../../assets/branding", "logo.rc")
}
