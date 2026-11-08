//! The battery ring of one tile: a filled arc with round ends, a solid circle for a connected device that reports no level, or a dashed circle for one that is off.

use std::f32::consts::{FRAC_PI_2, TAU};

use crate::gfx::{Canvas, Color, Mask, Path, Point, Rect};

use super::layout::RING_DIAMETER;

const RING_RADIUS: f32 = 25.0;
const STROKE: f32 = 4.0;
const MARGIN: u32 = 1;
const STEPS: u32 = 50;
const POINTS_PER_TURN: f32 = 96.0;
const DASH_COUNT: u32 = 20;
/// The share of each dash period that is drawn (3 of every 8 px in the design).
const DASH_FILL: f32 = 3.0 / 8.0;
const DASH_ARC_POINTS: u32 = 4;

pub(super) const TRACK: Color = Color::rgb(0.173, 0.173, 0.188);
/// The ring of a device without a level: solid when it is on and does not report, dashed when it is off.
pub(super) const NO_LEVEL: Color = Color::rgb(0.227, 0.227, 0.267);

pub(super) enum Fill {
    Level {
        fraction: f32,
        color: Color,
    },
    /// A full ring: the device is on but Windows reports no battery.
    Solid,
    Dashed,
}

pub(super) fn draw(canvas: &Canvas, center: Point, fill: &Fill, opacity: f32) {
    let scale = canvas.scale();
    let side = (RING_DIAMETER * scale).ceil() as u32 + 2 * MARGIN;
    let left = (center.x - side as f32 / 2.0).round();
    let top = (center.y - side as f32 / 2.0).round();
    let area = Rect::new(left, top, left + side as f32, top + side as f32);
    let middle = Point::new(side as f32 / 2.0, side as f32 / 2.0);
    let (radius, half_width) = (RING_RADIUS * scale, STROKE * scale / 2.0);
    match *fill {
        Fill::Solid => {
            let key = ("devices-solid", side);
            if let Some(image) = canvas.cached_image(key, || {
                let figures = Path::polyline(&arc_points(middle, radius, 1.0)).flatten();
                Mask::stroke(side, side, &figures, half_width).tinted(NO_LEVEL.r, NO_LEVEL.g, NO_LEVEL.b)
            }) {
                canvas.image(&image, area, 0.0, opacity);
            }
        }
        Fill::Dashed => {
            let key = ("devices-dashed", side);
            if let Some(image) = canvas.cached_image(key, || {
                let figures: Vec<_> = dash_polygons(middle, radius, half_width)
                    .iter()
                    .flat_map(|points| Path::polyline(points).close().flatten())
                    .collect();
                Mask::fill(side, side, &figures).tinted(NO_LEVEL.r, NO_LEVEL.g, NO_LEVEL.b)
            }) {
                canvas.image(&image, area, 0.0, opacity);
            }
        }
        Fill::Level { fraction, color } => {
            let track_key = ("devices-track", side);
            if let Some(track) = canvas.cached_image(track_key, || {
                let figures = Path::polyline(&arc_points(middle, radius, 1.0)).flatten();
                Mask::stroke(side, side, &figures, half_width).tinted(TRACK.r, TRACK.g, TRACK.b)
            }) {
                canvas.image(&track, area, 0.0, opacity);
            }
            let step = step_for(fraction);
            if step == 0 {
                return;
            }
            let arc_key = ("devices-arc", side, step, color.rgb_bytes());
            if let Some(arc) = canvas.cached_image(arc_key, || {
                let points = arc_points(middle, radius, step as f32 / STEPS as f32);
                let figures = Path::polyline(&points).flatten();
                Mask::stroke(side, side, &figures, half_width).tinted(color.r, color.g, color.b)
            }) {
                canvas.image(&arc, area, 0.0, opacity * color.a);
            }
        }
    }
}

/// A level above zero always shows at least one step, so a nearly empty battery is not an empty ring.
pub(super) fn step_for(fraction: f32) -> u32 {
    let fraction = fraction.clamp(0.0, 1.0);
    let step = (fraction * STEPS as f32).round() as u32;
    if fraction > 0.0 { step.max(1) } else { step }
}

fn arc_points(center: Point, radius: f32, fraction: f32) -> Vec<Point> {
    let count = ((POINTS_PER_TURN * fraction).ceil() as usize).max(1);
    (0..=count)
        .map(|index| {
            let angle = -FRAC_PI_2 + TAU * fraction * index as f32 / count as f32;
            Point::new(center.x + radius * angle.cos(), center.y + radius * angle.sin())
        })
        .collect()
}

/// One closed outline per dash: the outer edge forward, the inner edge back.
fn dash_polygons(center: Point, radius: f32, half_width: f32) -> Vec<Vec<Point>> {
    let period = TAU / DASH_COUNT as f32;
    (0..DASH_COUNT)
        .map(|dash| {
            let start = -FRAC_PI_2 + dash as f32 * period;
            let along = |reach: f32, step: u32| {
                let angle = start + period * DASH_FILL * step as f32 / DASH_ARC_POINTS as f32;
                Point::new(center.x + reach * angle.cos(), center.y + reach * angle.sin())
            };
            let outer = (0..=DASH_ARC_POINTS).map(|step| along(radius + half_width, step));
            let inner = (0..=DASH_ARC_POINTS).rev().map(|step| along(radius - half_width, step));
            outer.chain(inner).collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_snap_to_steps_and_a_low_level_keeps_one_step() {
        assert_eq!(step_for(0.0), 0);
        assert_eq!(step_for(1.0), STEPS);
        assert_eq!(step_for(0.82), 41);
        assert_eq!(step_for(0.001), 1);
        assert_eq!(step_for(3.0), STEPS);
    }

    #[test]
    fn the_arc_starts_at_the_top_and_a_half_turn_ends_at_the_bottom() {
        let points = arc_points(Point::new(10.0, 10.0), 5.0, 0.5);
        assert!((points[0].x - 10.0).abs() < 0.001 && (points[0].y - 5.0).abs() < 0.001);
        let end = points.last().unwrap();
        assert!((end.x - 10.0).abs() < 0.001 && (end.y - 15.0).abs() < 0.001);
    }

    #[test]
    fn the_dashed_ring_has_twenty_dashes_inside_the_ring_band() {
        let polygons = dash_polygons(Point::new(0.0, 0.0), 25.0, 2.0);
        assert_eq!(polygons.len(), 20);
        for point in polygons.iter().flatten() {
            let distance = point.x.hypot(point.y);
            assert!((23.0 - 0.01..=27.0 + 0.01).contains(&distance), "{distance}");
        }
    }
}
