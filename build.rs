include!("scripts/icon_resource.rs");

fn main() -> Result<(), Box<dyn Error>> {
    compile_icon_resource("assets/branding", "logo.rc")
}
