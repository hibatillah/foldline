//! Reads the logo of the app a notification came from: Windows' own answer first, then the logo file the app's package
//! declares in its manifest (some packages, Edge among them, make `GetLogo` fail although the file is there).

use std::{fs, path::Path};

use windows::{
    ApplicationModel::AppInfo,
    Foundation::Size,
    Graphics::Imaging::{
        BitmapAlphaMode, BitmapDecoder, BitmapPixelFormat, BitmapTransform, ColorManagementMode, ExifOrientationMode,
    },
    Storage::Streams::{DataWriter, IRandomAccessStream, InMemoryRandomAccessStream},
    core::{Interface, Result},
};

use crate::{gfx::ImageData, platform::app_icon};

const LOGO_PIXELS: u32 = 64;
/// Manifest attributes that name the app's logo, the small tile first.
const LOGO_ATTRIBUTES: [&str; 2] = ["Square44x44Logo", "Square150x150Logo"];

pub(super) fn read_logo(app_info: &AppInfo, app_id: &str) -> Result<ImageData> {
    let from_windows = app_info
        .DisplayInfo()
        .and_then(|info| info.GetLogo(Size { Width: LOGO_PIXELS as f32, Height: LOGO_PIXELS as f32 }))
        .and_then(|reference| decode(&reference.OpenReadAsync()?.join()?.cast()?));
    match from_windows {
        Ok(image) => Ok(image),
        Err(error) => {
            let package_directory = app_icon::package_install_path(app_id).ok_or_else(|| error.clone())?;
            let file = manifest_logo_file(Path::new(&package_directory)).ok_or(error)?;
            let stream = InMemoryRandomAccessStream::new()?;
            let writer = DataWriter::CreateDataWriter(&stream)?;
            writer.WriteBytes(&fs::read(file)?)?;
            writer.StoreAsync()?.join()?;
            writer.DetachStream()?;
            stream.Seek(0)?;
            decode(&stream.cast()?)
        }
    }
}

fn decode(stream: &IRandomAccessStream) -> Result<ImageData> {
    let decoder = BitmapDecoder::CreateAsync(stream)?.join()?;
    let transform = BitmapTransform::new()?;
    transform.SetScaledWidth(LOGO_PIXELS)?;
    transform.SetScaledHeight(LOGO_PIXELS)?;
    let pixels = decoder
        .GetPixelDataTransformedAsync(
            BitmapPixelFormat::Bgra8,
            BitmapAlphaMode::Premultiplied,
            &transform,
            ExifOrientationMode::IgnoreExifOrientation,
            ColorManagementMode::DoNotColorManage,
        )?
        .join()?
        .DetachPixelData()?;
    Ok(ImageData::new(LOGO_PIXELS, LOGO_PIXELS, pixels.to_vec()))
}

/// The path of the logo file the package's manifest names, or of its closest scaled variant on disk.
fn manifest_logo_file(package_directory: &Path) -> Option<String> {
    let manifest = fs::read_to_string(package_directory.join("AppxManifest.xml")).ok()?;
    let relative = LOGO_ATTRIBUTES.iter().find_map(|attribute| attribute_value(&manifest, attribute))?;
    let relative = relative.replace('/', "\\");
    let exact = package_directory.join(&relative);
    if exact.is_file() {
        return Some(exact.to_string_lossy().into_owned());
    }
    let parent = exact.parent()?;
    let file_name = exact.file_name()?.to_string_lossy().into_owned();
    let names: Vec<String> = fs::read_dir(parent)
        .ok()?
        .filter_map(|entry| Some(entry.ok()?.file_name().to_string_lossy().into_owned()))
        .collect();
    let variant = scaled_variant(&file_name, &names)?;
    Some(parent.join(variant).to_string_lossy().into_owned())
}

/// The value of the first `name="value"` attribute in the manifest.
fn attribute_value(manifest: &str, name: &str) -> Option<String> {
    let marker = format!("{name}=\"");
    let start = manifest.find(&marker)? + marker.len();
    let length = manifest[start..].find('"')?;
    Some(manifest[start..start + length].to_string()).filter(|value| !value.is_empty())
}

/// Packages ship `Logo.scale-200.png` for a manifest that says `Logo.png`; picks the 200 % one, else the 100 % one,
/// else the first.
fn scaled_variant(file_name: &str, names: &[String]) -> Option<String> {
    let (stem, extension) = file_name.rsplit_once('.')?;
    let prefix = format!("{stem}.");
    let suffix = format!(".{extension}");
    let mut variants: Vec<&String> = names
        .iter()
        .filter(|name| name.len() > prefix.len() + suffix.len() && name.starts_with(&prefix) && name.ends_with(&suffix))
        .collect();
    variants.sort();
    ["scale-200", "scale-100"]
        .iter()
        .find_map(|scale| variants.iter().find(|name| name.contains(scale)))
        .or(variants.first())
        .map(|name| (*name).clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_logo_attribute_is_read_from_the_manifest() {
        let manifest =
            r#"<VisualElements DisplayName="Edge" Square150x150Logo="Logo.png" Square44x44Logo="SmallLogo.png"/>"#;
        assert_eq!(attribute_value(manifest, "Square44x44Logo").as_deref(), Some("SmallLogo.png"));
        assert_eq!(attribute_value(manifest, "Square71x71Logo"), None);
    }

    #[test]
    fn an_empty_attribute_is_no_logo() {
        assert_eq!(attribute_value(r#"<V Square44x44Logo=""/>"#, "Square44x44Logo"), None);
    }

    #[test]
    fn the_scaled_variant_prefers_200_percent_then_100_percent() {
        let names: Vec<String> =
            ["Logo.scale-100.png", "Logo.scale-200.png", "Other.scale-200.png"].map(String::from).to_vec();
        assert_eq!(scaled_variant("Logo.png", &names).as_deref(), Some("Logo.scale-200.png"));
        assert_eq!(scaled_variant("Logo.png", &names[..1]).as_deref(), Some("Logo.scale-100.png"));
    }

    #[test]
    fn a_name_that_only_shares_a_prefix_is_not_a_variant() {
        let names = vec!["Logos.png".to_string(), "Logo.png".to_string()];
        assert_eq!(scaled_variant("Logo.png", &names), None);
    }
}
