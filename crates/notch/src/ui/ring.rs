use super::Icon;
use crate::gfx::{Canvas, Color, ImageData, Mask, Path, Point};

const LINE_WIDTH: f32 = 2.0;
const STEPS: u32 = 48;
const POINTS_PER_TURN: f32 = 96.0;
const TRACK_ALPHA: f32 = 0.22;
/// The charging bolt is half the ring's height; the ring's hole is 0.75 of its diameter at most.
const BOLT_HEIGHT_RATIO: f32 = 0.5;
/// `Icon::Bolt` covers rows 3 to 21 of its 24 grid, so its square is larger than the bolt.
const BOLT_GRID_SHARE: f32 = 18.0 / 24.0;

/// Progress arcs are cached; center and diameter are in the canvas's space (device pixels outside `in_units`).
pub struct Ring {
    pub center: Point,
    pub diameter: f32,
}

impl Ring {
    pub fn draw(&self, canvas: &Canvas, fraction: f32, color: Color, opacity: f32) {
        let (side, area) = canvas.pixel_square(self.center, self.diameter);
        let diameter = self.diameter * canvas.raster_scale();
        let stroke_width = LINE_WIDTH * diameter / 16.0;
        let radius = (diameter - stroke_width) / 2.0;
        let half_width = stroke_width / 2.0;
        let step = step_for(fraction);

        let track_key = ("ring-track", side);
        if let Some(track) = canvas.cached_image(track_key, || arc_image(side, radius, half_width, 1.0, Color::WHITE)) {
            canvas.image(&track, area, 0.0, opacity * TRACK_ALPHA);
        }
        if step == 0 {
            return;
        }
        let arc_key = ("ring-arc", side, step, color.rgb_bytes());
        let arc_fraction = step as f32 / STEPS as f32;
        if let Some(arc) = canvas.cached_image(arc_key, || arc_image(side, radius, half_width, arc_fraction, color)) {
            canvas.image(&arc, area, 0.0, opacity * color.a);
        }
    }

    /// A solid lightning bolt in the middle of the ring, for a battery that is charging.
    pub fn draw_bolt(&self, canvas: &Canvas, color: Color, opacity: f32) {
        Icon::Bolt.draw(canvas, self.center, bolt_icon_size(self.diameter), color, opacity);
    }
}

fn bolt_icon_size(diameter: f32) -> f32 {
    diameter * BOLT_HEIGHT_RATIO / BOLT_GRID_SHARE
}

fn arc_image(side: u32, radius: f32, half_width: f32, fraction: f32, color: Color) -> ImageData {
    let middle = Point::new(side as f32 / 2.0, side as f32 / 2.0);
    let figures = Path::polyline(&arc_points(middle, radius, fraction)).flatten();
    Mask::stroke(side, side, &figures, half_width).tinted(color.r, color.g, color.b)
}

fn step_for(fraction: f32) -> u32 {
    (fraction.clamp(0.0, 1.0) * STEPS as f32).round() as u32
}

fn arc_points(center: Point, radius: f32, fraction: f32) -> Vec<Point> {
    let fraction = fraction.clamp(0.0, 1.0);
    let count = ((POINTS_PER_TURN * fraction).ceil() as usize).max(1);
    (0..=count)
        .map(|index| {
            let angle = -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * fraction * index as f32 / count as f32;
            Point::new(center.x + radius * angle.cos(), center.y + radius * angle.sin())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::test_support::{SURFACE_WIDTH, render};

    #[test]
    fn the_arc_starts_at_the_top_of_the_ring() {
        let points = arc_points(Point::new(10.0, 10.0), 5.0, 0.5);
        assert!((points[0].x - 10.0).abs() < 0.001);
        assert!((points[0].y - 5.0).abs() < 0.001);
    }

    #[test]
    fn half_a_turn_ends_at_the_bottom_and_a_quarter_at_the_right() {
        let half = arc_points(Point::new(10.0, 10.0), 5.0, 0.5);
        let end = half.last().unwrap();
        assert!((end.x - 10.0).abs() < 0.001 && (end.y - 15.0).abs() < 0.001);
        let quarter = arc_points(Point::new(10.0, 10.0), 5.0, 0.25);
        let end = quarter.last().unwrap();
        assert!((end.x - 15.0).abs() < 0.001 && (end.y - 10.0).abs() < 0.001);
    }

    #[test]
    fn a_full_turn_closes_the_ring() {
        let full = arc_points(Point::new(0.0, 0.0), 4.0, 1.0);
        let (first, last) = (full[0], full[full.len() - 1]);
        assert!((first.x - last.x).abs() < 0.001 && (first.y - last.y).abs() < 0.001);
    }

    /// Device-pixel bounds `(left, top, right, bottom)` of the pixels at least a quarter covered, and their distance
    /// from `center` at the farthest.
    fn covered(pixels: &[u8], center: Point) -> ((f32, f32, f32, f32), f32) {
        let stride = SURFACE_WIDTH as usize * 4;
        let (mut bounds, mut farthest) = ((f32::MAX, f32::MAX, f32::MIN, f32::MIN), 0.0_f32);
        for (index, pixel) in pixels.chunks(4).enumerate() {
            if pixel[3] < 64 {
                continue;
            }
            let (x, y) = ((index * 4 % stride / 4) as f32, (index * 4 / stride) as f32);
            bounds = (bounds.0.min(x), bounds.1.min(y), bounds.2.max(x + 1.0), bounds.3.max(y + 1.0));
            farthest = farthest.max((x + 0.5 - center.x).hypot(y + 0.5 - center.y));
        }
        (bounds, farthest)
    }

    #[test]
    fn the_charging_bolt_is_centered_half_the_ring_tall_and_inside_its_hole() {
        let (origin, center, diameter) = (Point::new(7.3, 3.2), Point::new(40.4, 20.1), 14.0);
        for scale in [1.0, 1.25, 1.5] {
            let pixels = render(scale, |canvas| {
                canvas.in_units(origin, |canvas| {
                    Ring { center, diameter }.draw_bolt(canvas, Color::WHITE, 1.0);
                });
            });
            let device_center = Point::new(origin.x + center.x * scale, origin.y + center.y * scale);
            let ((left, top, right, bottom), farthest) = covered(&pixels, device_center);
            assert!(right > left, "the bolt covers some pixels at {scale}");
            let height = bottom - top;
            assert!((height - diameter * 0.5 * scale).abs() <= 1.5, "height {height} at {scale}");
            let (middle_x, middle_y) = ((left + right) / 2.0, (top + bottom) / 2.0);
            assert!((middle_x - device_center.x).abs() <= 1.0, "x {middle_x} against {} at {scale}", device_center.x);
            assert!((middle_y - device_center.y).abs() <= 1.0, "y {middle_y} against {} at {scale}", device_center.y);
            let hole_radius = (diameter / 2.0 - LINE_WIDTH * diameter / 16.0) * scale;
            assert!(farthest <= hole_radius, "the bolt reaches {farthest}, the hole ends at {hole_radius} at {scale}");
        }
    }

    #[test]
    fn the_bolt_takes_the_color_it_is_given() {
        let pixels = render(1.0, |canvas| {
            Ring { center: Point::new(30.0, 30.0), diameter: 14.0 }.draw_bolt(canvas, Color::rgb(0.0, 1.0, 0.0), 1.0);
        });
        let strongest = pixels.chunks(4).max_by_key(|pixel| pixel[3]).expect("pixels");
        assert!(strongest[3] > 200 && strongest[1] > 200 && strongest[0] < 20 && strongest[2] < 20, "{strongest:?}");
    }

    #[test]
    fn fractions_snap_to_a_limited_set_of_steps() {
        assert_eq!(step_for(0.0), 0);
        assert_eq!(step_for(1.0), STEPS);
        assert_eq!(step_for(2.0), STEPS);
        assert_eq!(step_for(-1.0), 0);
        assert_eq!(step_for(0.5), STEPS / 2);
    }
}
