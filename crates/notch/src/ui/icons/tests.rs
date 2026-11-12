use super::{paths::Paint, *};
use crate::gfx::Path;

const ALL: [Icon; 31] = [
    Icon::Lyrics,
    Icon::Previous,
    Icon::Play,
    Icon::Pause,
    Icon::Next,
    Icon::Speaker,
    Icon::SpeakerMuted,
    Icon::Sun,
    Icon::Bolt,
    Icon::Battery,
    Icon::BatteryLow,
    Icon::Microphone,
    Icon::Camera,
    Icon::Copy,
    Icon::Bell,
    Icon::Clock,
    Icon::Folder,
    Icon::Calendar,
    Icon::Moon,
    Icon::Cloud,
    Icon::Bluetooth,
    Icon::CapsLock,
    Icon::Gear,
    Icon::Mouse,
    Icon::Keyboard,
    Icon::Headphones,
    Icon::Gamepad,
    Icon::Pen,
    Icon::CircleCheck,
    Icon::Pin,
    Icon::Download,
];

#[test]
fn every_icon_stays_inside_its_square() {
    let (center, size) = (Point::new(100.0, 50.0), 40.0);
    for icon in ALL {
        let (path, _) = icon.path(center, size);
        for point in path.points() {
            assert!((point.x - center.x).abs() <= size / 2.0 && (point.y - center.y).abs() <= size / 2.0, "{point:?}");
        }
    }
}

#[test]
fn solid_icons_are_filled_and_line_icons_are_stroked() {
    let paint = |icon: Icon| icon.path(Point::default(), 24.0).1;
    for icon in [Icon::Play, Icon::Pause, Icon::Previous, Icon::Next, Icon::Bolt] {
        assert_eq!(paint(icon), Paint::Fill);
    }
    for icon in
        ALL.into_iter().filter(|icon| ![Icon::Play, Icon::Pause, Icon::Previous, Icon::Next, Icon::Bolt].contains(icon))
    {
        assert_eq!(paint(icon), Paint::Line);
    }
}

#[test]
fn muting_the_speaker_replaces_its_waves_with_a_cross() {
    let count = |icon: Icon| icon.path(Point::default(), 24.0).0.points().count();
    assert_ne!(count(Icon::Speaker), count(Icon::SpeakerMuted));
}

#[test]
fn a_low_battery_shows_fewer_bars_than_a_battery() {
    let count = |icon: Icon| icon.path(Point::default(), 24.0).0.points().count();
    assert!(count(Icon::BatteryLow) < count(Icon::Battery));
}

#[test]
fn the_gear_is_a_pointy_top_hexagon_around_a_ring_drawn_as_a_line() {
    let (path, paint) = Icon::Gear.path(Point::default(), 24.0);
    assert_eq!(paint, Paint::Line);
    let (top, bottom) =
        path.points().fold((f32::MAX, f32::MIN), |(low, high), point| (low.min(point.y), high.max(point.y)));
    assert!(top < -9.0 && bottom > 9.0, "the hexagon keeps a vertex at the top and the bottom: {top} {bottom}");
    let farthest = path.points().map(|point| point.x.hypot(point.y)).fold(0.0, f32::max);
    assert!(farthest <= 10.01, "the hexagon stays inside the square with room for the stroke");
    let ring: Vec<f32> = path.points().map(|point| point.x.hypot(point.y)).filter(|&distance| distance < 5.0).collect();
    let on_curve = ring.iter().filter(|&&distance| (distance - 3.0).abs() < 0.01).count();
    assert!(on_curve >= 4, "the ring of radius 3 sits at the center: {ring:?}");
}

#[test]
fn the_moon_is_a_crescent_with_a_plus_shaped_star_at_its_top_right() {
    let (path, paint) = Icon::Moon.path(Point::default(), 24.0);
    assert_eq!(paint, Paint::Line);
    let figures = path.flatten();
    assert_eq!(figures.len(), 3, "the crescent and the two bars of the star");
    for bar in &figures[1..] {
        assert!(bar.points.iter().all(|point| point.x >= 5.0 && point.y <= -5.0), "{bar:?}");
    }
    let crescent = &figures[0].points;
    let left = crescent.iter().map(|point| point.x).fold(f32::MAX, f32::min);
    let bottom = crescent.iter().map(|point| point.y).fold(f32::MIN, f32::max);
    assert!(left <= -8.9 && bottom >= 8.9, "the crescent keeps the big circle: {left} {bottom}");
    let first = crescent.first().expect("a start");
    let last = crescent.last().expect("an end");
    assert!((first.x - last.x).hypot(first.y - last.y) < 0.01, "the crescent ends where it starts");
}

#[test]
fn next_mirrors_previous() {
    let (previous, _) = Icon::Previous.path(Point::default(), 24.0);
    let (next, _) = Icon::Next.path(Point::default(), 24.0);
    let (previous_extent, next_extent) = (extent(&previous), extent(&next));
    assert!((previous_extent.0 + next_extent.1).abs() < 1.0, "{previous_extent:?} {next_extent:?}");
}

fn extent(path: &Path) -> (f32, f32) {
    path.points().fold((f32::MAX, f32::MIN), |(low, high), point| (low.min(point.x), high.max(point.x)))
}

/// Width and height of what a stroked line icon covers at `size`: its flattened points plus the stroke on each side.
fn stroked_extent(icon: Icon, size: f32) -> (f32, f32) {
    let figures = icon.path(Point::default(), size).0.flatten();
    let points: Vec<Point> = figures.into_iter().flat_map(|figure| figure.points).collect();
    let span = |axis: fn(&Point) -> f32| {
        let (low, high) =
            points.iter().map(axis).fold((f32::MAX, f32::MIN), |(low, high), along| (low.min(along), high.max(along)));
        high - low + LINE_WIDTH * size / GRID
    };
    (span(|point| point.x), span(|point| point.y))
}

#[test]
fn the_pin_has_a_head_above_a_needle_that_reaches_down_the_middle() {
    let (path, paint) = Icon::Pin.path(Point::default(), 24.0);
    assert_eq!(paint, Paint::Line);
    let figures = path.flatten();
    assert_eq!(figures.len(), 2, "the head and the needle");
    let needle = &figures[1].points;
    assert!(needle.iter().all(|point| point.x.abs() < 0.01 && point.y >= 5.0), "{needle:?}");
    let head_bottom = figures[0].points.iter().map(|point| point.y).fold(f32::MIN, f32::max);
    assert!(head_bottom <= 5.01, "the head ends where the needle starts: {head_bottom}");
}

#[test]
fn at_the_header_size_the_pin_shows_about_9_by_12_and_no_taller_than_the_gear() {
    let (width, height) = stroked_extent(Icon::Pin, 13.0);
    assert!((width - 8.7).abs() < 0.3 && (height - 11.9).abs() < 0.3, "{width} x {height}");
    let (_, gear_height) = stroked_extent(Icon::Gear, 16.0);
    assert!(height <= gear_height, "{height} against the gear's {gear_height}");
}

#[test]
fn the_download_icon_draws_an_arrow_above_a_tray_in_the_standard_size() {
    assert_eq!(Icon::Download.size(), 18.0);
    let (path, paint) = Icon::Download.path(Point::default(), 24.0);
    assert_eq!(paint, Paint::Line);
    let figures = path.flatten();
    assert_eq!(figures.len(), 3, "the shaft, the arrowhead and the tray");
    let arrow = figures[..2].iter().flat_map(|figure| figure.points.iter());
    let arrow_bottom = arrow.map(|point| point.y).fold(f32::MIN, f32::max);
    let tray_top = figures[2].points.iter().map(|point| point.y).fold(f32::MAX, f32::min);
    assert!(arrow_bottom < tray_top, "the arrow ends above the tray's top edge: {arrow_bottom} {tray_top}");
}

#[test]
fn the_download_icon_rasterizes_to_visible_pixels_at_18_pixels() {
    let image = Icon::Download.render(18.0, 18, Color::WHITE);
    assert_eq!((image.width, image.height), (18, 18));
    assert!(image.bgra_premultiplied.chunks(4).any(|pixel| pixel[3] > 0), "the icon leaves some pixel covered");
}
