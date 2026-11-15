//! The map beside the city search: a drawn street grid and river with a pin, the saved city's name and coordinates.
//! A picture, not a real map: it shows that one place serves every page.

use gpui::{AnyElement, App, Hsla, SharedString, canvas, div, point, prelude::*, px, rgba};
use places::Location;
use store::Settings;

use crate::{i18n::tr, illustration, picture::color};

const MAP_HEIGHT: f32 = 132.0;
const MAP_FILL: u32 = 0x18223a;
const STREET: u32 = 0x263352;
const STREET_WIDTH: f32 = 4.0;
const RIVER: u32 = 0x2c4a86;
const RIVER_WIDTH: f32 = 9.0;
const PIN_SIZE: f32 = 22.0;
const PIN_DOT: f32 = 8.0;
const LABEL_SIZE: f32 = 11.0;

/// "0.51° N, 101.45° E" for a latitude and longitude in degrees.
fn coordinates_text(latitude: f64, longitude: f64) -> String {
    let north_south = if latitude < 0.0 { "S" } else { "N" };
    let east_west = if longitude < 0.0 { "W" } else { "E" };
    format!("{:.2}° {north_south}, {:.2}° {east_west}", latitude.abs(), longitude.abs())
}

/// The city name from a saved label ("Pekanbaru, ID" gives "Pekanbaru").
fn city_name(label: &str) -> &str {
    label.split_once(',').map_or(label, |(name, _)| name).trim()
}

fn street(vertical: bool, at: f32) -> gpui::Div {
    let line = div().absolute().bg(color(STREET));
    if vertical {
        line.top_0().bottom_0().left(gpui::relative(at)).w(px(STREET_WIDTH))
    } else {
        line.left_0().right_0().top(gpui::relative(at)).h(px(STREET_WIDTH))
    }
}

/// A river that bends across the lower part of the map.
fn river() -> impl IntoElement {
    canvas(
        |_bounds, _window, _cx| (),
        |bounds, _, window, _cx| {
            let mut builder = gpui::PathBuilder::stroke(px(RIVER_WIDTH));
            let steps = 40;
            for step in 0..=steps {
                let progress = step as f32 / steps as f32;
                let x = bounds.origin.x + bounds.size.width * progress;
                let wave = (progress * std::f32::consts::PI * 1.2 + 0.4).sin();
                let y = bounds.origin.y + bounds.size.height * (0.74 - 0.12 * progress + 0.05 * wave);
                if step == 0 {
                    builder.move_to(point(x, y));
                } else {
                    builder.line_to(point(x, y));
                }
            }
            if let Ok(path) = builder.build() {
                window.paint_path(path, color(RIVER));
            }
        },
    )
    .absolute()
    .size_full()
}

/// The pin: an accent disc with a white dot and a point under it.
fn pin(accent: Hsla) -> AnyElement {
    let point_under = canvas(
        |_bounds, _window, _cx| (),
        move |bounds, _, window, _cx| {
            let mut builder = gpui::PathBuilder::fill();
            let (left, top, width) = (bounds.origin.x, bounds.origin.y, bounds.size.width);
            builder.move_to(point(left, top));
            builder.line_to(point(left + width, top));
            builder.line_to(point(left + width / 2.0, top + bounds.size.height));
            builder.close();
            if let Ok(path) = builder.build() {
                window.paint_path(path, accent);
            }
        },
    )
    .w(px(12.0))
    .h(px(9.0))
    .mt(px(-4.0));
    gpui_component::v_flex()
        .items_center()
        .child(
            div()
                .size(px(PIN_SIZE))
                .rounded_full()
                .bg(accent)
                .flex()
                .items_center()
                .justify_center()
                .child(div().size(px(PIN_DOT)).rounded_full().bg(color(0xffffff))),
        )
        .child(point_under)
        .into_any_element()
}

fn label_chip(text: impl Into<SharedString>) -> gpui::Div {
    div().px(px(7.0)).py(px(2.0)).rounded(px(5.0)).bg(rgba(0x0000008c)).text_size(px(LABEL_SIZE)).child(text.into())
}

pub fn city_map(cx: &App) -> AnyElement {
    let settings = Settings::load();
    let location = Location::read(&|key| settings.get_string(key).map(str::to_owned));
    let accent = crate::palette::accent(cx);
    let (name, coordinates) = match &location {
        Some(location) => (
            SharedString::from(city_name(&location.label).to_owned()),
            Some(coordinates_text(location.latitude, location.longitude)),
        ),
        None => (SharedString::from(tr!("general.city_none")), None),
    };
    illustration::scene(MAP_HEIGHT)
        .child(div().absolute().size_full().bg(color(MAP_FILL)))
        .child(street(true, 0.35))
        .child(street(true, 0.65))
        .child(street(false, 0.36))
        .child(street(false, 0.72))
        .child(river())
        .children(
            location
                .is_some()
                .then(|| div().absolute().top(px(18.0)).left(gpui::relative(0.5)).ml(px(-11.0)).child(pin(accent))),
        )
        .child(div().absolute().left(px(8.0)).bottom(px(7.0)).child(label_chip(name)))
        .children(coordinates.map(|text| {
            div().absolute().right(px(8.0)).bottom(px(7.0)).child(label_chip(text).text_color(color(0xb8c0d4)))
        }))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinates_name_the_hemisphere_and_drop_the_sign() {
        assert_eq!(coordinates_text(0.5071, 101.4478), "0.51° N, 101.45° E");
        assert_eq!(coordinates_text(-6.2088, -106.8456), "6.21° S, 106.85° W");
    }

    #[test]
    fn the_map_names_the_city_without_its_country() {
        assert_eq!(city_name("Pekanbaru, ID"), "Pekanbaru");
        assert_eq!(city_name("Lagos"), "Lagos");
    }
}
