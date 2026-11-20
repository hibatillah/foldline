//! The window's own icons: Lucide drawings the component library's bundle does not carry (see `assets/icons`), and the
//! asset source that serves them next to the library's own icons.

use gpui::{AssetSource, Result, SharedString};
use gpui_component_assets::Assets;
use std::borrow::Cow;

/// Declares this window's own icons, one line each: `NAME = "file-stem"` serves `assets/icons/<file-stem>.svg` at
/// `icons/<file-stem>.svg`. A doc comment above the line says what the icon is.
macro_rules! own_icons {
    ($($(#[$doc:meta])* $name:ident = $stem:literal),+ $(,)?) => {
        $($(#[$doc])* pub const $name: &str = concat!("icons/", $stem, ".svg");)+

        const OWN_ICONS: &[(&str, &str)] =
            &[$(($name, include_str!(concat!("../assets/icons/", $stem, ".svg")))),+];
    };
}

own_icons! {
    /// The notch's own settings icon: a pointy-top hexagon around a ring, drawn like `Icon::Gear` in the notch.
    GENERAL = "hex-nut",
    /// Lucide "layers".
    PAGES = "layers",
    /// Lucide "puzzle".
    BEHAVIOR = "puzzle",
    /// Lucide "calendar-days".
    CALENDAR = "calendar-days",
    /// Lucide "moon-star".
    SHALAT = "moon-star",
    /// Lucide "chart-column".
    STATS = "chart-column",
    /// Lucide "wrench".
    ADVANCED = "wrench",
    /// Lucide "cloud-drizzle".
    WEATHER = "cloud-drizzle",
    /// Lucide "info".
    ABOUT = "info",
    /// Lucide "play", the media page.
    PLAY = "play",
    /// Lucide "timer".
    TIMER = "timer",
    /// Lucide "square-check", the todo page.
    TODO = "square-check",
    /// Lucide "bluetooth", the connected devices page.
    BLUETOOTH = "bluetooth",
    /// Lucide "clipboard".
    CLIPBOARD = "clipboard",
    /// Lucide "keyboard", the hint about the number keys.
    KEYBOARD = "keyboard",
    /// Lucide "mouse-pointer-2", the pointer in the pictures.
    POINTER = "mouse-pointer-2",
    /// Lucide "volume-2", the volume message in the Behavior pictures.
    VOLUME = "volume-2",
    /// Lucide "triangle-alert", the icon of a warning box.
    WARNING = "triangle-alert",
}

/// Lucide "inbox", the file shelf; the component library's bundle serves it.
pub const INBOX: &str = "icons/inbox.svg";
/// Lucide "file"; the component library's bundle serves it.
pub const FILE: &str = "icons/file.svg";

/// Serves this window's icons first and the component library's bundle for everything else.
pub struct WindowAssets;

impl AssetSource for WindowAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        match OWN_ICONS.iter().find(|(own_path, _)| *own_path == path) {
            Some((_, svg)) => Ok(Some(Cow::Borrowed(svg.as_bytes()))),
            None => Assets.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = Assets.list(path)?;
        paths.extend(
            OWN_ICONS
                .iter()
                .filter(|(own_path, _)| own_path.starts_with(path))
                .map(|(own_path, _)| SharedString::from(*own_path)),
        );
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_own_icon_is_served_as_a_24_grid_svg_that_follows_the_text_color() {
        for (path, _) in OWN_ICONS {
            let bytes = WindowAssets.load(path).expect("load").expect("served");
            let svg = String::from_utf8(bytes.into_owned()).expect("utf-8");
            assert!(svg.contains("viewBox=\"0 0 24 24\"") && svg.contains("currentColor"), "{path}");
        }
    }

    #[test]
    fn every_svg_file_in_the_assets_folder_is_declared_once() {
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/icons");
        let mut files: Vec<String> = std::fs::read_dir(folder)
            .expect("assets/icons")
            .flatten()
            .map(|entry| format!("icons/{}", entry.file_name().to_string_lossy()))
            .collect();
        let mut declared: Vec<String> = OWN_ICONS.iter().map(|(path, _)| (*path).to_owned()).collect();
        files.sort();
        declared.sort();
        assert_eq!(files, declared);
    }

    #[test]
    fn the_librarys_own_icons_are_still_served() {
        for path in ["icons/chevron-right.svg", INBOX, FILE] {
            assert!(WindowAssets.load(path).expect("load").is_some(), "{path}");
        }
    }

    #[test]
    fn the_hex_nut_is_a_six_cornered_outline_around_a_ring_of_radius_2_8() {
        let svg = WindowAssets.load(GENERAL).expect("load").expect("served");
        let svg = String::from_utf8(svg.into_owned()).expect("utf-8");
        assert_eq!(svg.matches('A').count(), 6);
        assert!(svg.contains("r=\"2.8\""));
    }
}
