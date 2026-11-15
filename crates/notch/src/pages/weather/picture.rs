//! The little pictures of the sky: the Lucide icon of a sky, tinted by what it brings.

use super::symbol::Symbol;
use crate::gfx::{Canvas, Color, Point};

const SUN: Color = Color::rgb(1.0, 0.753, 0.471);
const CLOUD: Color = Color::rgb(0.769, 0.769, 0.8);
const CLOUD_NOW: Color = Color::rgb(0.902, 0.902, 0.922);
const DROP: Color = Color::rgb(0.455, 0.753, 0.988);
const BOLT: Color = Color::rgb(1.0, 0.831, 0.231);
const SNOW: Color = Color::rgb(0.906, 0.941, 1.0);

/// The color of the whole icon: warm for the sun, blue for rain, yellow for thunder, white for snow, else the cloud grey.
fn tint(symbol: Symbol, is_now: bool) -> Color {
    match symbol {
        Symbol::Clear | Symbol::PartlyCloudy => SUN,
        Symbol::Drizzle | Symbol::Rain | Symbol::HeavyRain | Symbol::RainShowers | Symbol::RainShowersNight => DROP,
        Symbol::Thunderstorm => BOLT,
        Symbol::Snow => SNOW,
        Symbol::ClearNight | Symbol::PartlyCloudyNight | Symbol::Cloudy | Symbol::Fog => {
            if is_now {
                CLOUD_NOW
            } else {
                CLOUD
            }
        }
    }
}

/// Draws `symbol` in a square of `size` units around `center`; `is_now` brightens the grey of the highlighted cell.
pub(super) fn draw_symbol(canvas: &Canvas, symbol: Symbol, center: Point, size: f32, is_now: bool, opacity: f32) {
    symbol.icon().draw(canvas, center, size, tint(symbol, is_now), opacity);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_highlighted_cell_gets_a_brighter_cloud() {
        let plain = tint(Symbol::Cloudy, false);
        let now = tint(Symbol::Cloudy, true);
        assert!(now.r > plain.r);
    }

    #[test]
    fn rain_snow_and_thunder_each_have_their_own_tint() {
        let tints = [Symbol::Rain, Symbol::Snow, Symbol::Thunderstorm, Symbol::Clear, Symbol::Cloudy].map(|symbol| {
            let color = tint(symbol, false);
            (color.r.to_bits(), color.g.to_bits(), color.b.to_bits())
        });
        for (index, tint) in tints.iter().enumerate() {
            assert!(!tints[index + 1..].contains(tint));
        }
    }
}
