//! The little pictures of the sky: Lucide-style line icons in a few colors, drawn from path data and cached.

use super::symbol::Symbol;
use crate::{
    gfx::{Canvas, Color, Point},
    ui::{Stroke, draw_strokes},
};

const LINE_WIDTH: f32 = 2.0;

const SUN: Color = Color::rgb(1.0, 0.753, 0.471);
const CLOUD: Color = Color::rgb(0.769, 0.769, 0.8);
const CLOUD_NOW: Color = Color::rgb(0.902, 0.902, 0.922);
const DROP: Color = Color::rgb(0.455, 0.753, 0.988);
const BOLT: Color = Color::rgb(1.0, 0.831, 0.231);
const SNOW: Color = Color::rgb(0.906, 0.941, 1.0);
const MIST: Color = Color::rgb(0.549, 0.549, 0.58);

const SUN_RAYS: &str =
    "M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4";
const SUN_DISC: &str = "M8 12a4 4 0 1 0 8 0a4 4 0 1 0-8 0";
const PARTIAL_SUN_RAYS: &str = "M8 2.5v1.5M3.2 4.7l1 1M1.5 9.5H3M12.8 4.7l-1 1";
const PARTIAL_SUN_DISC: &str = "M5 12.6A4 4 0 1 1 11.8 8.4";
const LOW_CLOUD: &str = "M9 20h8a4 4 0 0 0 0-8 5.5 5.5 0 0 0-10.4 1.8A3.1 3.1 0 0 0 9 20z";
const MID_CLOUD: &str = "M7 19h10a4.5 4.5 0 0 0 .5-9A6 6 0 0 0 6 11.5 3.75 3.75 0 0 0 7 19z";
const HIGH_CLOUD: &str = "M7 15h10a4.5 4.5 0 0 0 .5-9A6 6 0 0 0 6 7.5 3.75 3.75 0 0 0 7 15z";
const MOON: &str = "M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5z";
const PARTIAL_MOON: &str = "M5.2 11.4A4.6 4.6 0 0 1 8.6 3.4a3.6 3.6 0 0 0 4.7 4.7";
const RAIN_DROPS: &str = "M8 18l-1 3M12 18l-1 3M16 18l-1 3";
const DRIZZLE_DROPS: &str = "M9 18l-1 3M14 18l-1 3";
const BOLT_SHAPE: &str = "M13 15.5l-2.5 3.5h3.5l-2.5 3.5";
const SNOW_FLAKES: &str = "M8 19h.01M12 19h.01M16 19h.01M10 22h.01M14 22h.01";
const MIST_LINES: &str = "M6 19h12M9 22h6";

/// The layers of `symbol`, drawn in order; `is_now` brightens the cloud for the highlighted cell.
fn layers(symbol: Symbol, is_now: bool) -> Vec<Stroke> {
    let cloud = if is_now { CLOUD_NOW } else { CLOUD };
    let stroke = Stroke::new;
    match symbol {
        Symbol::Clear => vec![stroke(SUN_DISC, SUN), stroke(SUN_RAYS, SUN)],
        Symbol::ClearNight => vec![stroke(MOON, cloud)],
        Symbol::PartlyCloudy => {
            vec![stroke(PARTIAL_SUN_RAYS, SUN), stroke(PARTIAL_SUN_DISC, SUN), stroke(LOW_CLOUD, cloud)]
        }
        Symbol::PartlyCloudyNight => vec![stroke(PARTIAL_MOON, SUN), stroke(LOW_CLOUD, cloud)],
        Symbol::Cloudy => vec![stroke(MID_CLOUD, cloud)],
        Symbol::Fog => vec![stroke(HIGH_CLOUD, cloud), stroke(MIST_LINES, MIST)],
        Symbol::Drizzle => vec![stroke(HIGH_CLOUD, cloud), stroke(DRIZZLE_DROPS, DROP)],
        Symbol::Rain => vec![stroke(HIGH_CLOUD, cloud), stroke(RAIN_DROPS, DROP)],
        Symbol::Thunderstorm => vec![stroke(HIGH_CLOUD, cloud), stroke(BOLT_SHAPE, BOLT)],
        Symbol::Snow => vec![stroke(HIGH_CLOUD, cloud), stroke(SNOW_FLAKES, SNOW)],
    }
}

/// Draws `symbol` in a square of `size` device pixels around `center`.
pub(super) fn draw_symbol(canvas: &Canvas, symbol: Symbol, center: Point, size: f32, is_now: bool, opacity: f32) {
    draw_strokes(canvas, &layers(symbol, is_now), center, size, LINE_WIDTH, opacity);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sky_has_a_picture_and_wet_ones_have_a_cloud_and_water() {
        for symbol in [
            Symbol::Clear,
            Symbol::ClearNight,
            Symbol::PartlyCloudy,
            Symbol::PartlyCloudyNight,
            Symbol::Cloudy,
            Symbol::Fog,
            Symbol::Drizzle,
            Symbol::Rain,
            Symbol::Thunderstorm,
            Symbol::Snow,
        ] {
            let count = layers(symbol, false).len();
            assert!(count >= 1, "{symbol:?}");
            assert!(!symbol.is_wet() || count == 2, "{symbol:?} needs its cloud and its water");
        }
    }

    #[test]
    fn the_highlighted_cell_gets_a_brighter_cloud() {
        let plain = layers(Symbol::Cloudy, false)[0].color;
        let now = layers(Symbol::Cloudy, true)[0].color;
        assert!(now.r > plain.r);
    }
}
