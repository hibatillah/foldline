//! The little pictures of the sky, built from circles and rounded rectangles so they need no path geometry.

use super::symbol::Symbol;
use crate::gfx::{Canvas, Color, Point, Rect};

const SUN: Color = Color::rgb(1.0, 0.8, 0.25);
const MOON: Color = Color::rgb(0.92, 0.92, 1.0);
const CLOUD: Color = Color::rgb(0.78, 0.8, 0.86);
const DROP: Color = Color::rgb(0.3, 0.67, 0.97);
const SNOW: Color = Color::rgb(0.9, 0.93, 1.0);
const BOLT: Color = Color::rgb(1.0, 0.82, 0.2);
const MIST: Color = Color::rgb(0.6, 0.62, 0.68);
const RAY_COUNT: usize = 8;

/// Draws `symbol` in a square of `size` device pixels around `center`; `backdrop` is the color behind it, which the
/// moon's bite is cut with.
pub(super) fn draw_symbol(canvas: &Canvas, symbol: Symbol, center: Point, size: f32, backdrop: Color, opacity: f32) {
    let circle = |x: f32, y: f32, radius: f32, color: Color| {
        canvas.fill_circle(Point::new(center.x + x * size, center.y + y * size), radius * size, color.faded(opacity));
    };
    let bar = |left: f32, top: f32, right: f32, bottom: f32, color: Color| {
        let rect =
            Rect::new(center.x + left * size, center.y + top * size, center.x + right * size, center.y + bottom * size);
        canvas.fill_rounded_rect(rect, rect.width().min(rect.height()) / 2.0, color.faded(opacity));
    };
    let sun = |x: f32, y: f32, scale: f32| {
        circle(x, y, 0.2 * scale, SUN);
        for ray in 0..RAY_COUNT {
            let angle = ray as f32 / RAY_COUNT as f32 * std::f32::consts::TAU;
            circle(x + 0.34 * scale * angle.cos(), y + 0.34 * scale * angle.sin(), 0.04 * scale, SUN);
        }
    };
    let cloud = |x: f32, y: f32, scale: f32| {
        bar(x - 0.34 * scale, y - 0.04 * scale, x + 0.34 * scale, y + 0.2 * scale, CLOUD);
        circle(x - 0.12 * scale, y - 0.02 * scale, 0.16 * scale, CLOUD);
        circle(x + 0.1 * scale, y - 0.08 * scale, 0.2 * scale, CLOUD);
    };
    match symbol {
        Symbol::Clear => sun(0.0, 0.0, 1.0),
        Symbol::ClearNight => {
            circle(0.0, 0.0, 0.27, MOON);
            circle(0.13, -0.1, 0.23, backdrop);
        }
        Symbol::PartlyCloudy => {
            sun(-0.14, -0.14, 0.72);
            cloud(0.06, 0.1, 0.85);
        }
        Symbol::PartlyCloudyNight => {
            circle(-0.14, -0.14, 0.17, MOON);
            circle(-0.06, -0.2, 0.15, backdrop);
            cloud(0.06, 0.1, 0.85);
        }
        Symbol::Cloudy => cloud(0.0, 0.0, 1.0),
        Symbol::Fog => {
            cloud(0.0, -0.14, 0.85);
            bar(-0.3, 0.2, 0.3, 0.25, MIST);
            bar(-0.2, 0.32, 0.4, 0.37, MIST);
        }
        Symbol::Drizzle => {
            cloud(0.0, -0.14, 0.85);
            for x in [-0.12, 0.1] {
                bar(x - 0.025, 0.2, x + 0.025, 0.3, DROP);
            }
        }
        Symbol::Rain => {
            cloud(0.0, -0.14, 0.85);
            for x in [-0.2, 0.0, 0.2] {
                bar(x - 0.03, 0.2, x + 0.03, 0.36, DROP);
            }
        }
        Symbol::Thunderstorm => {
            cloud(0.0, -0.16, 0.85);
            bar(-0.02, 0.14, 0.06, 0.28, BOLT);
            bar(-0.08, 0.26, 0.0, 0.38, BOLT);
        }
        Symbol::Snow => {
            cloud(0.0, -0.14, 0.85);
            for x in [-0.2, 0.0, 0.2] {
                circle(x, 0.28, 0.04, SNOW);
            }
        }
    }
}
