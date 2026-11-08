//! The window's own icons: Lucide drawings the component library's bundle does not carry (see `assets/icons`), and the
//! asset source that serves them next to the library's own icons.

use gpui::{AssetSource, Result, SharedString};
use gpui_component_assets::Assets;
use std::borrow::Cow;

/// The notch's own settings icon: a pointy-top hexagon around a ring, drawn like `Icon::Gear` in the notch.
pub const GENERAL: &str = "icons/hex-nut.svg";
/// Lucide "layers".
pub const PAGES: &str = "icons/layers.svg";
/// Lucide "puzzle".
pub const BEHAVIOR: &str = "icons/puzzle.svg";
/// Lucide "calendar-days".
pub const CALENDAR: &str = "icons/calendar-days.svg";
/// Lucide "moon-star".
pub const SHALAT: &str = "icons/moon-star.svg";
/// Lucide "chart-column".
pub const STATS: &str = "icons/chart-column.svg";
/// Lucide "wrench".
pub const ADVANCED: &str = "icons/wrench.svg";
/// Lucide "cloud-drizzle".
pub const WEATHER: &str = "icons/cloud-drizzle.svg";
/// Lucide "folder".
pub const FOLDER: &str = "icons/folder.svg";
/// Lucide "info".
pub const ABOUT: &str = "icons/info.svg";

const OWN_ICONS: [(&str, &str); 10] = [
    (GENERAL, include_str!("../assets/icons/hex-nut.svg")),
    (PAGES, include_str!("../assets/icons/layers.svg")),
    (BEHAVIOR, include_str!("../assets/icons/puzzle.svg")),
    (CALENDAR, include_str!("../assets/icons/calendar-days.svg")),
    (SHALAT, include_str!("../assets/icons/moon-star.svg")),
    (STATS, include_str!("../assets/icons/chart-column.svg")),
    (ADVANCED, include_str!("../assets/icons/wrench.svg")),
    (WEATHER, include_str!("../assets/icons/cloud-drizzle.svg")),
    (FOLDER, include_str!("../assets/icons/folder.svg")),
    (ABOUT, include_str!("../assets/icons/info.svg")),
];

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
    fn the_librarys_own_icons_are_still_served() {
        assert!(WindowAssets.load("icons/chevron-right.svg").expect("load").is_some());
    }

    #[test]
    fn the_hex_nut_is_a_six_cornered_outline_around_a_ring_of_radius_2_8() {
        let svg = WindowAssets.load(GENERAL).expect("load").expect("served");
        let svg = String::from_utf8(svg.into_owned()).expect("utf-8");
        assert_eq!(svg.matches('A').count(), 6);
        assert!(svg.contains("r=\"2.8\""));
    }
}
