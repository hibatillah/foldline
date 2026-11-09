use super::bitmap_square::bitmap_square;
use crate::gfx::{Canvas, Color, ImageData, Mask, Path, Point};

const LINE_WIDTH: f32 = 2.0;
const STEPS: u32 = 48;
const POINTS_PER_TURN: f32 = 96.0;
const TRACK_ALPHA: f32 = 0.22;

/// Progress arcs are cached; center and diameter are device pixels.
pub struct Ring {
    pub center: Point,
    pub diameter: f32,
}

impl Ring {
    pub fn draw(&self, canvas: &Canvas, fraction: f32, color: Color, opacity: f32) {
        let (side, area) = bitmap_square(self.center, self.diameter);
        let stroke_width = LINE_WIDTH * self.diameter / 16.0;
        let radius = (self.diameter - stroke_width) / 2.0;
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

    #[test]
    fn fractions_snap_to_a_limited_set_of_steps() {
        assert_eq!(step_for(0.0), 0);
        assert_eq!(step_for(1.0), STEPS);
        assert_eq!(step_for(2.0), STEPS);
        assert_eq!(step_for(-1.0), 0);
        assert_eq!(step_for(0.5), STEPS / 2);
    }
}
