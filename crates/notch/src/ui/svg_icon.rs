//! Line icons drawn from SVG path data on a 24 unit grid, one color per layer.

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

/// Draws the layers in order, centered on `center`; `size` is the edge of the 24 unit grid in the canvas's space (device
/// pixels outside `in_units`) and `line_width` is in grid units.
pub fn draw_strokes(canvas: &Canvas, strokes: &[Stroke], center: Point, size: f32, line_width: f32, opacity: f32) {
    let (side, area) = canvas.pixel_square(center, size);
    let device_size = size * canvas.raster_scale();
    for stroke in strokes {
        let key = ("svg-stroke", stroke.data, side, stroke.color.rgb_bytes(), line_width.to_bits());
        let Some(brush) = canvas.cached_image(key, || render(stroke, device_size, side, line_width)) else { continue };
        canvas.image(&brush, area, 0.0, opacity * stroke.color.a);
    }
}

fn render(stroke: &Stroke, size: f32, side: u32, line_width: f32) -> ImageData {
    let unit = size / GRID;
    let corner = side as f32 / 2.0 - size / 2.0;
    let figures = Path::from_svg(stroke.data, Point::new(corner, corner), unit).flatten();
    Mask::stroke(side, side, &figures, line_width * unit / 2.0).tinted(stroke.color.r, stroke.color.g, stroke.color.b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::test_support::render as render_surface;

    const STROKES: [Stroke; 1] = [Stroke::new("M4 12h16M12 4v16", Color::WHITE)];

    #[test]
    fn an_icon_drawn_in_units_matches_the_same_icon_scaled_by_hand() {
        let in_units = render_surface(1.5, |canvas| {
            canvas.in_units(Point::new(7.3, 3.2), |canvas| {
                draw_strokes(canvas, &STROKES, Point::new(20.4, 20.1), 18.0, 2.0, 1.0);
            });
        });
        let by_hand = render_surface(1.5, |canvas| {
            draw_strokes(canvas, &STROKES, Point::new(7.3 + 20.4 * 1.5, 3.2 + 20.1 * 1.5), 27.0, 2.0, 1.0);
        });
        let inked =
            in_units.as_chunks::<4>().0.iter().zip(by_hand.as_chunks::<4>().0).filter(|(a, b)| a[3] > 0 || b[3] > 0);
        let (total, differing) =
            inked.fold((0_usize, 0_usize), |(total, differing), (a, b)| (total + 1, differing + usize::from(a != b)));
        assert!(total > 0, "nothing was drawn");
        assert!(differing * 100 <= total, "{differing} of {total} inked pixels differ");
    }
}
