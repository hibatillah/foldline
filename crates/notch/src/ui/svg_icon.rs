//! Line icons drawn from SVG path data on a 24 unit grid, one color per layer.

use super::bitmap_square::bitmap_square;
use crate::gfx::{Canvas, Color, ImageData, Mask, Path, Point};

const GRID: f32 = 24.0;

/// One stroked layer of an icon.
#[derive(Clone, Copy)]
pub struct Stroke {
    pub data: &'static str,
    pub color: Color,
}

impl Stroke {
    pub const fn new(data: &'static str, color: Color) -> Self {
        Self { data, color }
    }
}

/// Draws the layers in order, centered on `center`; `size` is the edge of the 24 unit grid in device pixels and
/// `line_width` is in grid units.
pub fn draw_strokes(canvas: &Canvas, strokes: &[Stroke], center: Point, size: f32, line_width: f32, opacity: f32) {
    let (side, area) = bitmap_square(center, size);
    for stroke in strokes {
        let key = ("svg-stroke", stroke.data, side, stroke.color.rgb_bytes(), line_width.to_bits());
        let Some(brush) = canvas.cached_image(key, || render(stroke, size, side, line_width)) else { continue };
        canvas.image(&brush, area, 0.0, opacity * stroke.color.a);
    }
}

fn render(stroke: &Stroke, size: f32, side: u32, line_width: f32) -> ImageData {
    let unit = size / GRID;
    let corner = side as f32 / 2.0 - size / 2.0;
    let figures = Path::from_svg(stroke.data, Point::new(corner, corner), unit).flatten();
    Mask::stroke(side, side, &figures, line_width * unit / 2.0).tinted(stroke.color.r, stroke.color.g, stroke.color.b)
}
