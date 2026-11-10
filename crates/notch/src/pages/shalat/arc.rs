//! The day as one sun arc through the five prayers, filled up to now; the curve is a sketch, not the sun's height.

use salah::{DateTime, Utc};

use crate::gfx::{Canvas, Color, ImageData, Mask, Path, Point, Rect};
use crate::ui::palette::{MINT_GREEN, SURFACE_OUTLINE};

/// Where each of the five prayers sits on the arc, in DIP from the top of the 86 high page.
const POINT_HEIGHTS: [f32; 5] = [50.0, 14.0, 23.4, 40.3, 50.0];
const BASELINE: f32 = 50.0;
const TOP: f32 = 14.0;
pub(super) const PRAYER_COUNT: usize = 5;
/// Where the five prayers are in `Schedule::today`, which also holds sunrise.
pub(super) const PRAYER_SLOTS: [usize; PRAYER_COUNT] = [0, 2, 3, 4, 5];

const LINE_WIDTH: f32 = 2.0;
const GLOW_RADIUS: f32 = 11.0;
const GLOW_ALPHA: f32 = 0.55;
const FUTURE_FILL: Color = Color::WHITE;
const PAST_FILL_ALPHA: f32 = 0.30;
const FUTURE_FILL_ALPHA: f32 = 0.07;
const AHEAD_RING: Color = Color::rgb(0.353, 0.353, 0.4);
const SUN_GLOW: Color = Color::rgb(1.0, 0.831, 0.231);
/// The unclipped images: the gray and green areas, then the gray and green lines.
const LAYER_COUNT: u8 = 4;

/// How a prayer's marker looks: done, the one coming next, or still ahead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Marker {
    Passed,
    Next,
    Ahead,
}

/// Where now falls on the day's arc.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Position {
    BeforeFajr,
    /// Prayer slots from Fajr (0) to Isha (4), with the fraction between two prayers.
    During(f32),
    AfterIsha,
}

pub(super) fn position(today: &[DateTime<Utc>; 6], now: DateTime<Utc>) -> Position {
    let times = PRAYER_SLOTS.map(|slot| today[slot]);
    if now < times[0] {
        return Position::BeforeFajr;
    }
    if now >= times[PRAYER_COUNT - 1] {
        return Position::AfterIsha;
    }
    let index = (0..PRAYER_COUNT - 1).find(|&index| now < times[index + 1]).unwrap_or(0);
    let span = (times[index + 1] - times[index]).num_seconds().max(1) as f32;
    Position::During(index as f32 + (now - times[index]).num_seconds() as f32 / span)
}

type Pair = (f32, f32);

/// The prayer markers' x positions and the curve's four control points of `segment`, in DIP for a chart `width` wide.
pub(super) fn marker_x(index: usize, width: f32) -> f32 {
    (index as f32 + 0.5) * width / PRAYER_COUNT as f32
}

/// A cubic through every marker with each handle a sixth of the way to the marker after (Catmull-Rom).
fn segment(index: usize, width: f32) -> [Pair; 4] {
    let at = |marker: usize| {
        let marker = marker.min(PRAYER_COUNT - 1);
        (marker_x(marker, width), POINT_HEIGHTS[marker])
    };
    let (before, start, end, after) = (at(index.saturating_sub(1)), at(index), at(index + 1), at(index + 2));
    [
        start,
        (start.0 + (end.0 - before.0) / 6.0, start.1 + (end.1 - before.1) / 6.0),
        (end.0 - (after.0 - start.0) / 6.0, end.1 - (after.1 - start.1) / 6.0),
        end,
    ]
}

fn cubic(points: &[Pair; 4], amount: f32) -> Pair {
    let rest = 1.0 - amount;
    let weights =
        [rest * rest * rest, 3.0 * rest * rest * amount, 3.0 * rest * amount * amount, amount * amount * amount];
    let x = points.iter().zip(weights).map(|(point, weight)| point.0 * weight).sum();
    let y = points.iter().zip(weights).map(|(point, weight)| point.1 * weight).sum();
    (x, y)
}

/// The arc's point at prayer slot `slot` (0 to 4, fractions between), in DIP.
pub(super) fn point_at(slot: f32, width: f32) -> Pair {
    let index = (slot.floor() as usize).min(PRAYER_COUNT - 2);
    cubic(&segment(index, width), (slot - index as f32).clamp(0.0, 1.0))
}

fn curve(width: f32, raster_scale: f32) -> Path {
    let to_pixels = |point: Pair| Point::new(point.0 * raster_scale, point.1 * raster_scale);
    let mut path = Path::new().move_to(to_pixels(segment(0, width)[0]));
    for index in 0..PRAYER_COUNT - 1 {
        let [_, first, second, end] = segment(index, width);
        path = path.curve_to(to_pixels(first), to_pixels(second), to_pixels(end));
    }
    path
}

fn area(width: f32, raster_scale: f32) -> Path {
    let (first, last) = (marker_x(0, width) * raster_scale, marker_x(PRAYER_COUNT - 1, width) * raster_scale);
    let mut path = curve(width, raster_scale);
    path = path.line_to(Point::new(last, BASELINE * raster_scale)).line_to(Point::new(first, BASELINE * raster_scale));
    path.close()
}

fn line_image(width: f32, height: u32, raster_scale: f32, color: Color) -> ImageData {
    let figures = curve(width, raster_scale).flatten();
    Mask::stroke(width_pixels(width, raster_scale), height, &figures, LINE_WIDTH * raster_scale / 2.0)
        .tinted(color.r, color.g, color.b)
}

fn fill_image(width: f32, height: u32, raster_scale: f32, color: Color, top_alpha: f32) -> ImageData {
    let figures = area(width, raster_scale).flatten();
    let (top, bottom) = (TOP * raster_scale, BASELINE * raster_scale);
    Mask::fill(width_pixels(width, raster_scale), height, &figures).painted(|_, row| {
        let amount = ((row as f32 + 0.5 - top) / (bottom - top)).clamp(0.0, 1.0);
        color.with_alpha(top_alpha * (1.0 - amount))
    })
}

fn width_pixels(width: f32, raster_scale: f32) -> u32 {
    (width * raster_scale).round() as u32
}

/// The sun's glow in a `side` pixel square.
fn glow_image(side: u32, raster_scale: f32) -> ImageData {
    let radius = GLOW_RADIUS * raster_scale;
    let middle = side as f32 / 2.0;
    let mask = Mask::from_distance(side, side, |point| (point.x - middle).hypot(point.y - middle) - radius);
    mask.painted(|x, y| {
        let distance = ((x as f32 + 0.5 - middle).hypot(y as f32 + 0.5 - middle) / radius).min(1.0);
        SUN_GLOW.with_alpha(GLOW_ALPHA * (1.0 - distance))
    })
}

/// How far along the arc `position` has come, as an x in DIP of the chart, or `None` before Fajr.
pub(super) fn progress_x(position: Position, width: f32) -> Option<f32> {
    match position {
        Position::BeforeFajr => None,
        Position::During(slot) => Some(point_at(slot, width).0),
        Position::AfterIsha => Some(width),
    }
}

fn draw_marker(canvas: &Canvas, center: Point, marker: Marker, opacity: f32) {
    match marker {
        Marker::Passed => canvas.fill_circle(center, 3.5, MINT_GREEN.faded(opacity)),
        Marker::Next => {
            canvas.fill_circle(center, 7.0, MINT_GREEN.with_alpha(0.18).faded(opacity));
            canvas.fill_circle(center, 4.5, MINT_GREEN.faded(opacity));
        }
        Marker::Ahead => {
            canvas.fill_circle(center, 4.25, AHEAD_RING.faded(opacity));
            canvas.fill_circle(center, 2.75, Color::BLACK.faded(opacity));
        }
    }
}

/// Draws the arc into `chart` (units, as high as the page): the gray remainder, the green past and the sun.
pub(super) fn draw(canvas: &Canvas, chart: Rect, position: Position, markers: [Marker; PRAYER_COUNT], opacity: f32) {
    let raster_scale = canvas.raster_scale();
    let width = chart.width();
    let width_pixels = width_pixels(width, raster_scale);
    let height = (chart.height() * raster_scale).round() as u32;
    let rect = Rect::new(chart.left, chart.top, chart.left + width_pixels as f32 / raster_scale, chart.bottom);
    let key = |kind: u8| ("shalat-arc", kind, width_pixels, height);
    let split = progress_x(position, width).map_or(rect.left, |x| (chart.left + x).min(rect.right));
    let future = Rect { left: split, ..rect };
    let past = Rect { right: split, ..rect };
    for kind in 0..LAYER_COUNT {
        let clip = if kind % 2 == 0 { future } else { past };
        if clip.width() <= 0.0 {
            continue;
        }
        let build = || match kind {
            0 => fill_image(width, height, raster_scale, FUTURE_FILL, FUTURE_FILL_ALPHA),
            1 => fill_image(width, height, raster_scale, MINT_GREEN, PAST_FILL_ALPHA),
            2 => line_image(width, height, raster_scale, SURFACE_OUTLINE),
            _ => line_image(width, height, raster_scale, MINT_GREEN),
        };
        let Some(brush) = canvas.cached_image(key(kind), build) else { continue };
        canvas.clipped(clip, || canvas.image(&brush, rect, 0.0, opacity));
    }
    for (index, marker) in markers.into_iter().enumerate() {
        let (x, y) = point_at(index as f32, width);
        draw_marker(canvas, Point::new(chart.left + x, chart.top + y), marker, opacity);
    }
    if let Position::During(slot) = position {
        let (x, y) = point_at(slot, width);
        let center = Point::new(chart.left + x, chart.top + y);
        let (side, glow_rect) = canvas.pixel_square(center, GLOW_RADIUS * 2.0);
        if let Some(glow) = canvas.cached_image(("shalat-sun-glow", side), || glow_image(side, raster_scale)) {
            canvas.image(&glow, glow_rect, 0.0, opacity);
        }
        canvas.fill_circle(center, 6.75, Color::BLACK.faded(opacity));
        canvas.fill_circle(center, 5.25, SUN_GLOW.faded(opacity));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use salah::Duration;

    const WIDTH: f32 = 351.0;

    fn day() -> [DateTime<Utc>; 6] {
        let start = DateTime::parse_from_rfc3339("2026-11-05T00:00:00Z").unwrap().with_timezone(&Utc);
        [4, 6, 12, 15, 18, 19].map(|hour| start + Duration::hours(hour))
    }

    #[test]
    fn the_markers_sit_in_the_middle_of_five_equal_slots() {
        assert!((marker_x(0, WIDTH) - 35.1).abs() < 0.01);
        assert!((marker_x(4, WIDTH) - 315.9).abs() < 0.01);
    }

    #[test]
    fn the_arc_passes_through_every_marker() {
        for (index, height) in POINT_HEIGHTS.iter().enumerate() {
            let (x, y) = point_at(index as f32, WIDTH);
            assert!((x - marker_x(index, WIDTH)).abs() < 0.01 && (y - height).abs() < 0.01, "{index}");
        }
    }

    #[test]
    fn the_arc_matches_the_design_between_dhuhr_and_asr() {
        let (x, y) = point_at(1.1508, WIDTH);
        assert!((x - 115.8).abs() < 1.0 && (y - 12.9).abs() < 0.3, "{x} {y}");
    }

    #[test]
    fn the_sun_is_found_by_time_between_two_prayers() {
        let times = day();
        assert_eq!(position(&times, times[0] - Duration::minutes(1)), Position::BeforeFajr);
        assert_eq!(position(&times, times[5] + Duration::minutes(1)), Position::AfterIsha);
        let Position::During(slot) = position(&times, times[2] + Duration::minutes(90)) else { panic!() };
        assert!((slot - 1.5).abs() < 0.001, "{slot}");
        assert_eq!(position(&times, times[0]), Position::During(0.0));
    }

    #[test]
    fn progress_runs_from_nothing_before_fajr_to_the_whole_width_after_isha() {
        assert_eq!(progress_x(Position::BeforeFajr, WIDTH), None);
        assert_eq!(progress_x(Position::AfterIsha, WIDTH), Some(WIDTH));
        let half = progress_x(Position::During(2.0), WIDTH).unwrap();
        assert!((half - marker_x(2, WIDTH)).abs() < 0.01);
    }

    /// The first draw builds four chart images and the glow on the UI thread; run with `-- --ignored --nocapture`.
    #[test]
    #[ignore = "timing"]
    fn the_first_draw_builds_its_images_in_under_fifty_milliseconds_at_raster_scale_one_and_a_half() {
        let raster_scale = 1.5;
        let height = (86.0 * raster_scale) as u32;
        let started = std::time::Instant::now();
        fill_image(WIDTH, height, raster_scale, FUTURE_FILL, FUTURE_FILL_ALPHA);
        fill_image(WIDTH, height, raster_scale, MINT_GREEN, PAST_FILL_ALPHA);
        line_image(WIDTH, height, raster_scale, SURFACE_OUTLINE);
        line_image(WIDTH, height, raster_scale, MINT_GREEN);
        glow_image((GLOW_RADIUS * 2.0 * raster_scale).ceil() as u32 + 2, raster_scale);
        let elapsed = started.elapsed();
        println!("arc first draw at raster_scale 1.5: {elapsed:?}");
        assert!(elapsed < std::time::Duration::from_millis(50), "{elapsed:?}");
    }
}
